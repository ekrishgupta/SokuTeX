use std::collections::VecDeque;
use mupdf::{Document, Colorspace, Matrix, DisplayList};
use std::error::Error;
use dashmap::DashMap;
use std::sync::Arc;
use ahash::RandomState;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct TileRenderQueue {
    pub visible_tiles: VecDeque<(u16, u16)>, // Render immediately
    pub adjacent_tiles: VecDeque<(u16, u16)>, // Next priority
    pub offscreen_tiles: VecDeque<(u16, u16)>, // Low priority
}

impl TileRenderQueue {
    pub fn new() -> Self {
        Self {
            visible_tiles: VecDeque::new(),
            adjacent_tiles: VecDeque::new(),
            offscreen_tiles: VecDeque::new(),
        }
    }

    pub fn prioritize_tiles(&mut self, viewport: Rect, all_tiles: Vec<(u16, u16)>) {
        self.visible_tiles.clear();
        self.adjacent_tiles.clear();
        self.offscreen_tiles.clear();

        for (tx, ty) in all_tiles {
            let tile_rect = Rect {
                x: tx as f32 * 256.0,
                y: ty as f32 * 256.0,
                width: 256.0,
                height: 256.0,
            };

            if self.intersects(&viewport, &tile_rect) {
                self.visible_tiles.push_back((tx, ty));
            } else if self.is_adjacent(&viewport, &tile_rect) {
                self.adjacent_tiles.push_back((tx, ty));
            } else {
                self.offscreen_tiles.push_back((tx, ty));
            }
        }
    }

    fn intersects(&self, r1: &Rect, r2: &Rect) -> bool {
        r1.x < r2.x + r2.width &&
        r1.x + r1.width > r2.x &&
        r1.y < r2.y + r2.height &&
        r1.y + r1.height > r2.y
    }

    fn is_adjacent(&self, r1: &Rect, r2: &Rect) -> bool {
        let margin = 256.0;
        let expanded_r1 = Rect {
            x: r1.x - margin,
            y: r1.y - margin,
            width: r1.width + margin * 2.0,
            height: r1.height + margin * 2.0,
        };
        self.intersects(&expanded_r1, r2)
    }
}

use std::sync::Mutex;

struct SendDocument(Document);
unsafe impl Send for SendDocument {}

#[allow(dead_code)]
struct SendDisplayList(DisplayList);
unsafe impl Send for SendDisplayList {}

pub struct PdfRenderer {
    // Cache for rendered pixmaps: (revision, page, width, height)
    cache: Arc<DashMap<(u64, i32, u16, u16), Arc<Vec<u8>>, RandomState>>,
    // Cache for interpreted display lists to avoid re-parsing the page
    #[allow(dead_code)]
    dl_cache: DashMap<(u64, i32), Arc<SendDisplayList>, RandomState>,
    doc_cache: DashMap<u64, Arc<Mutex<SendDocument>>, RandomState>,
    pub render_queue: Mutex<TileRenderQueue>,
}

impl PdfRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cache: Arc::new(DashMap::with_hasher(RandomState::new())),
            dl_cache: DashMap::with_hasher(RandomState::new()),
            doc_cache: DashMap::with_hasher(RandomState::new()),
            render_queue: Mutex::new(TileRenderQueue::new()),
        })
    }

    pub fn prioritize_tiles(&self, viewport: Rect, all_tiles: Vec<(u16, u16)>) {
        if let Ok(mut queue) = self.render_queue.lock() {
            queue.prioritize_tiles(viewport, all_tiles);
        }
    }



    pub fn render_page(&self, pdf_data: &[u8], revision: u64, page_index: i32, width: u16, height: u16) -> Result<(Arc<Vec<u8>>, f32, f32), Box<dyn Error>> {
        // Create a unique key for this render
        let key = (revision, page_index, width, height);
        
        let document_arc = if let Some(doc) = self.doc_cache.get(&revision) {
            doc.value().clone()
        } else {
            let doc = Arc::new(Mutex::new(SendDocument(Document::from_bytes(pdf_data, "")?)));
            self.doc_cache.insert(revision, doc.clone());
            doc
        };

        // We need to load the page anyway to get bounds if not cached? 
        // Actually, let's keep it simple.
        let document = document_arc.lock().map_err(|_| "Mutex poisoned")?;
        let page = document.0.load_page(page_index)?;
        let bounds = page.bounds()?;
        let pw = bounds.width();
        let ph = bounds.height();

        if let Some(cached) = self.cache.get(&key) {
            return Ok((cached.value().clone(), pw, ph));
        }

        let scale_x = width as f32 / pw;
        let scale_y = height as f32 / ph;
        
        let matrix = Matrix::new_scale(scale_x, scale_y);
        let colorspace = Colorspace::device_rgb();
        
        // Argument 'false, false' for show_extras and show_annotations (depends on mupdf-rs version)
        let pixmap = page.to_pixmap(&matrix, &colorspace, false, false)?;
        let samples = pixmap.samples();
        
        // Convert to BGRA
        let mut bgra_samples = vec![0u8; (samples.len() / 3) * 4];
        for (src, dst) in samples.chunks_exact(3).zip(bgra_samples.chunks_exact_mut(4)) {
            dst[0] = src[2];
            dst[1] = src[1];
            dst[2] = src[0];
            dst[3] = 255;
        }
        
        let arc_samples = Arc::new(bgra_samples);
        self.cache.insert(key, arc_samples.clone());
        Ok((arc_samples, pw, ph))
    }
}
