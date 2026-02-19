use mupdf::{Document, Colorspace, Matrix};
use std::error::Error;
use dashmap::DashMap;
use std::sync::Arc;
use ahash::RandomState;

pub struct PdfRenderer {
    cache: Arc<DashMap<(u64, u16, u16, i32), Arc<Vec<u8>>, RandomState>>,
}

impl PdfRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cache: Arc::new(DashMap::with_hasher(RandomState::new())),
        })
    }

    pub fn render_page(&self, pdf_data: &[u8], revision: u64, page_index: i32, width: u16, height: u16) -> Result<Arc<Vec<u8>>, Box<dyn Error>> {
        // Create a unique key for this render
        let key = (revision, width, height, page_index);
        
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.value().clone());
        }

        let document = Document::from_bytes(pdf_data, "")?;
        let page = document.load_page(page_index)?;
        
        let bounds = page.bounds()?;
        let scale_x = width as f32 / bounds.width();
        let scale_y = height as f32 / bounds.height();
        
        let matrix = Matrix::new_scale(scale_x, scale_y);
        let colorspace = Colorspace::device_rgb();
        
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
        Ok(arc_samples)
    }
}

