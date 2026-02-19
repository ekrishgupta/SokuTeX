use mupdf::{Document, Colorspace, Matrix};
use std::error::Error;
use dashmap::DashMap;
use std::sync::Arc;

pub struct PdfRenderer {
    cache: Arc<DashMap<(String, u16, u16, i32), Vec<u8>>>,
}

impl PdfRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cache: Arc::new(DashMap::new()),
        })
    }

    pub fn render_page(&self, pdf_data: &[u8], page_index: i32, width: u16, height: u16) -> Result<Vec<u8>, Box<dyn Error>> {
        // Create a unique key for this render
        let data_hash = format!("{:x}", md5::compute(pdf_data));
        let key = (data_hash, width, height, page_index);
        
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
        let mut bgra_samples = Vec::with_capacity(width as usize * height as usize * 4);
        for i in (0..samples.len()).step_by(3) {
            if i + 2 < samples.len() {
                bgra_samples.push(samples[i+2]);
                bgra_samples.push(samples[i+1]);
                bgra_samples.push(samples[i]);
                bgra_samples.push(255);
            }
        }
        
        self.cache.insert(key, bgra_samples.clone());
        Ok(bgra_samples)
    }
}
