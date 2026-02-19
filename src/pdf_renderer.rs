use pdfium_render::prelude::*;

pub struct PdfRenderer {
    pdfium: Pdfium,
}

impl PdfRenderer {
    pub fn new() -> Result<Self, PdfiumError> {
        let pdfium = Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))?);
        Ok(Self { pdfium })
    }
}
