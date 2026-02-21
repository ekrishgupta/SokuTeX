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
}

impl PdfRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cache: Arc::new(DashMap::with_hasher(RandomState::new())),
            dl_cache: DashMap::with_hasher(RandomState::new()),
            doc_cache: DashMap::with_hasher(RandomState::new()),
        })
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
