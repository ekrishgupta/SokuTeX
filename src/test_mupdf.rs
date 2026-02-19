use mupdf::{Document, DisplayList};

fn main() {
    let dl: DisplayList;
    {
        let pdf_data = include_bytes!("../workspace_preview.pdf");
        let document = Document::from_bytes(pdf_data, "").unwrap();
        let page = document.load_page(0).unwrap();
        dl = page.to_display_list(false).unwrap();
    } // Document and Page dropped here
    
    // Can we still use dl?
    // Let's try to render it
    let cs = mupdf::colorspace::Colorspace::device_rgb();
    let matrix = mupdf::matrix::Matrix::new_scale(1.0, 1.0);
    let _pixmap = dl.to_pixmap(&matrix, &cs, false).unwrap();
    println!("Survived document drop!");
}
