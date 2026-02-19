#[allow(dead_code)]
pub struct BibEntry {
    pub key: String,
    pub author: String,
    pub title: String,
}

#[allow(dead_code)]
pub struct BibParser;

impl BibParser {
    #[allow(dead_code)]
    pub fn parse(_content: &str) -> Vec<BibEntry> {
        // Placeholder parser
        vec![]
    }
}
