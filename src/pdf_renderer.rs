use pdfium_render::prelude::*;

pub struct PdfRenderer {
    pdfium: Pdfium,
}

impl PdfRenderer {
    pub fn new() -> Result<Self, PdfiumError> {
        // Try to bind to system library or local, depending on setup.
        // For now, let's assume dynamic linking works or the dylib is present.
        let pdfium = Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?);
        Ok(Self { pdfium })
    }

    pub fn render_page(&self, pdf_data: &[u8], page_index: u16, width: u16, height: u16) -> Result<Vec<u8>, PdfiumError> {
        let document = self.pdfium.load_pdf_from_byte_slice(pdf_data, None)?;
        let pages = document.pages();
        let page = pages.get(page_index)?;
        
        let render_config = PdfRenderConfig::new()
            .set_target_width(width as i32)
            .set_target_height(height as i32)
            .set_clear_color(PdfColor::new(255, 255, 255, 255))
            .rotate_if_landscape(PdfPageRenderRotation::None, true);

        let bitmap = page.render_with_config(&render_config)?;

        // Extract bytes
        let bytes = bitmap.as_raw_bytes(); 
        Ok(bytes.to_vec()) 
    }
}
