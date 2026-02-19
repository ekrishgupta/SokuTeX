use mupdf::{Document, Context, Colorspace, Matrix, Pixmap};
use std::error::Error;

pub struct PdfRenderer {
    context: Context,
}

impl PdfRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let context = Context::new()?;
        Ok(Self { context })
    }

    pub fn render_page(&self, pdf_data: &[u8], page_index: i32, width: u16, height: u16) -> Result<Vec<u8>, Box<dyn Error>> {
        let document = Document::from_bytes(&self.context, pdf_data, None)?;
        let page = document.load_page(page_index)?;
        
        let bounds = page.bounds()?;
        let scale_x = width as f32 / bounds.width();
        let scale_y = height as f32 / bounds.height();
        
        // Use uniform scaling to maintain aspect ratio, or use specific width/height
        let matrix = Matrix::new_scale(scale_x, scale_y);
        let colorspace = Colorspace::device_rgb(&self.context)?;
        
        let pixmap = Pixmap::new_from_page(&page, &matrix, &colorspace, 0)?;
        
        // Pixmap samples are in RGB format
        let samples = pixmap.samples();
        
        // Convert RGB to BGRA if required by the texture format (wgpu setup used Bgra8Unorm)
        let mut bgra_samples = Vec::with_capacity(width as usize * height as usize * 4);
        for i in (0..samples.len()).step_by(3) {
            if i + 2 < samples.len() {
                bgra_samples.push(samples[i+2]); // B
                bgra_samples.push(samples[i+1]); // G
                bgra_samples.push(samples[i]);   // R
                bgra_samples.push(255);          // A
            }
        }
        
        Ok(bgra_samples)
    }
}
