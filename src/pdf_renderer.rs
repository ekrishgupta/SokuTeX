use mupdf::{Document, Colorspace, Matrix, DisplayList};
use std::error::Error;
use dashmap::DashMap;
use std::sync::Arc;
use ahash::RandomState;

use std::sync::Mutex;

struct SendDocument(Document);
unsafe impl Send for SendDocument {}

struct SendDisplayList(DisplayList);
unsafe impl Send for SendDisplayList {}

pub struct PdfRenderer {
    // Cache for rendered pixmaps: (revision, page, width, height)
    cache: Arc<DashMap<(u64, i32, u16, u16), Arc<Vec<u8>>, RandomState>>,
    // Cache for interpreted display lists to avoid re-parsing the page
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



    pub fn render_page(&self, pdf_data: &[u8], revision: u64, page_index: i32, width: u16, height: u16) -> Result<Arc<Vec<u8>>, Box<dyn Error>> {
        // Create a unique key for this render
        let key = (revision, page_index, width, height);
        
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.value().clone());
        }

        let document_arc = if let Some(doc) = self.doc_cache.get(&revision) {
            doc.value().clone()
        } else {
            let doc = Arc::new(Mutex::new(SendDocument(Document::from_bytes(pdf_data, "")?)));
            self.doc_cache.insert(revision, doc.clone());
            doc
        };

        // Get or create Display List
        let dl = if let Some(dl) = self.dl_cache.get(&(revision, page_index)) {
            dl.value().clone()
        } else {
            let document = document_arc.lock().map_err(|_| "Mutex poisoned")?;
            let page = document.0.load_page(page_index)?;
            // Argument 'true' to include annotations in the display list
            let dl = Arc::new(SendDisplayList(page.to_display_list(true)?));
            self.dl_cache.insert((revision, page_index), dl.clone());
            dl
        };

        let document = document_arc.lock().map_err(|_| "Mutex poisoned")?;
        let page = document.0.load_page(page_index)?;
        let bounds = page.bounds()?;
        
        let scale_x = width as f32 / bounds.width();
        let scale_y = height as f32 / bounds.height();
        let matrix = Matrix::new_scale(scale_x, scale_y);
        let colorspace = Colorspace::device_rgb();
        
        // Use the display list for much faster re-rendering if only zoom changed
        // DisplayList::to_pixmap(matrix, colorspace, alpha)
        let pixmap = dl.0.to_pixmap(&matrix, &colorspace, false)?;

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
        Ok(arc_samples)
    }
}

