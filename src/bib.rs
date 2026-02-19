use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct BibEntry {
    pub key: String,
    pub entry_type: String,
    pub author: Option<String>,
    pub title: Option<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
}

pub struct BibParser;

impl BibParser {
    pub fn parse(content: &str) -> Vec<BibEntry> {
        let mut entries = Vec::new();
        // Regex to match BibTeX entries: @type{key, fields...}
        let entry_re = Regex::new(r"@(\w+)\s*\{\s*([^,]+),([\s\S]*?)\n\}").unwrap();
        let field_re = Regex::new(r"(\w+)\s*=\s*(?:\{([\s\S]*?)\}|([^{},\s][^,]*))").unwrap();

        for cap in entry_re.captures_iter(content) {
            let entry_type = cap[1].to_lowercase();
            let key = cap[2].trim().to_string();
            let fields_content = &cap[3];

            let mut author = None;
            let mut title = None;
            let mut year = None;
            let mut journal = None;

            for f_cap in field_re.captures_iter(fields_content) {
                let name = f_cap[1].to_lowercase();
                let value = f_cap.get(2).or(f_cap.get(3)).map(|m: regex::Match| m.as_str().trim().to_string());

                match name.as_str() {
                    "author" => author = value,
                    "title" => title = value,
                    "year" => year = value,
                    "journal" => journal = value,
                    _ => {}
                }
            }

            entries.push(BibEntry {
                key,
                entry_type,
                author,
                title,
                year,
                journal,
            });
        }
        entries
    }
}
