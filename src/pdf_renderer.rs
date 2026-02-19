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
        
        // Render to a bitmap
        let bitmap = page.render(width, height, None)?;
        
        // Extract bytes (BGRA usually, might need conversion for wgpu RGBA)
        // Pdfium usually renders BGRA or BGRx on desktop.
        let bytes = bitmap.as_bytes();
        
        // Simple copy for now, we might need swizzling later.
        Ok(bytes.to_vec())
    }
}
