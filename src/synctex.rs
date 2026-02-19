use std::path::{Path, PathBuf};
use ahash::AHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use flate2::read::GzDecoder;

#[derive(Debug, Clone)]
pub struct SyncTexNode {
    pub tag: u32,
    pub line: u32,
    pub column: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub page: u32,
}

pub struct SyncTex {
    pub inputs: AHashMap<u32, PathBuf>,
    pub nodes: Vec<SyncTexNode>,
    pub unit: f32, // Unit scaling from SyncTeX (usually 72/65536 or similar)
}

impl SyncTex {
    pub fn new() -> Self {
        Self {
            inputs: AHashMap::new(),
            nodes: Vec::new(),
            unit: 1.0,
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let file = File::open(path)?;
        
        if path.extension().map_or(false, |ext| ext == "gz") {
            self.load_from_reader(BufReader::new(GzDecoder::new(file)))
        } else {
            self.load_from_reader(BufReader::new(file))
        }
    }

    pub fn load_from_reader<R: BufRead>(&mut self, reader: R) -> std::io::Result<()> {
        let mut current_page = 0;
        let mut in_content = false;

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() { continue; }

            if line.starts_with("Input:") {
                let parts: Vec<&str> = line["Input:".len()..].split(':').collect();
                if parts.len() >= 2 {
                    if let Ok(tag) = parts[0].parse() {
                        self.inputs.insert(tag, PathBuf::from(parts[1]));
                    }
                }
            } else if line.starts_with('{') {
                if let Ok(page) = line[1..].parse() {
                    current_page = page;
                    in_content = true;
                }
            } else if line.starts_with('}') {
                in_content = false;
            } else if in_content {
                let first_char = line.chars().next().unwrap_or(' ');
                if "[(vhxkg".contains(first_char) {
                    self.parse_node(&line, current_page);
                }
            } else if line.starts_with("Unit:") {
                if let Ok(unit) = line["Unit:".len()..].parse() {
                    self.unit = unit;
                }
            }
        }
        Ok(())
    }

    fn parse_node(&mut self, line: &str, page: u32) {
        let content = &line[1..];
        let parts: Vec<&str> = content.split(|c| c == ',' || c == ':').collect();
        
        if parts.len() >= 5 {
            let tag = parts[0].parse().unwrap_or(0);
            let line_num = parts[1].parse().unwrap_or(0);
            let x = parts[2].parse().unwrap_or(0.0);
            let y = parts[3].parse().unwrap_or(0.0);
            
            let mut width = 0.0;
            let mut height = 0.0;
            let mut depth = 0.0;
            
            if parts.len() >= 7 {
                width = parts[4].parse().unwrap_or(0.0);
                height = parts[5].parse().unwrap_or(0.0);
                if parts.len() >= 8 {
                    depth = parts[6].parse().unwrap_or(0.0);
                }
            }

            self.nodes.push(SyncTexNode {
                tag,
                line: line_num,
                column: 0,
                x,
                y,
                width,
                height,
                depth,
                page,
            });
        }
    }

    pub fn forward_sync(&self, target_line: u32, target_tag: u32) -> Option<&SyncTexNode> {
        self.nodes.iter()
            .filter(|n| n.tag == target_tag && n.line >= target_line)
            .min_by_key(|n| n.line)
    }

    pub fn backward_sync(&self, page: u32, x: f32, y: f32) -> Option<&SyncTexNode> {
        self.nodes.iter()
            .filter(|n| n.page == page)
            .filter(|n| {
                x >= n.x && x <= (n.x + n.width) &&
                y >= (n.y - n.height) && y <= (n.y + n.depth)
            })
            .next()
            .or_else(|| {
                self.nodes.iter()
                    .filter(|n| n.page == page)
                    .min_by_key(|n| {
                        let dx = x - n.x;
                        let dy = y - n.y;
                        (dx * dx + dy * dy) as i32
                    })
            })
    }
}


