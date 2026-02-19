use regex::Regex;
use ahash::AHashSet;
use crate::vfs::Vfs;

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub children: Vec<DependencyNode>,
}

use std::sync::OnceLock;

static DEP_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct DependencyScanner;

impl DependencyScanner {
    pub fn scan(root_file: &str, vfs: &Vfs) -> DependencyNode {
        let mut visited = AHashSet::new();
        Self::scan_recursive(root_file, vfs, &mut visited)
    }

    fn scan_recursive(file_name: &str, vfs: &Vfs, visited: &mut AHashSet<String>) -> DependencyNode {
        let mut node = DependencyNode {
            name: file_name.to_string(),
            children: Vec::new(),
        };

        if visited.contains(file_name) {
            return node;
        }
        visited.insert(file_name.to_string());

        if let Some(content_bytes) = vfs.read_file(file_name) {
            let content = String::from_utf8_lossy(&content_bytes);
            let re = DEP_REGEX.get_or_init(|| Regex::new(r"\\(?:input|include|bibliography|usepackage)\{([^}]*)\}").unwrap());
            
            for cap in re.captures_iter(&content) {
                let matched_cmd = cap.get(0).unwrap().as_str();
                let sub_files = cap[1].to_string();
                
                for part in sub_files.split(',') {
                    let mut path = part.trim().to_string();
                    if path.is_empty() { continue; }

                    if matched_cmd.contains("bibliography") {
                        if !path.ends_with(".bib") { path.push_str(".bib"); }
                    } else if matched_cmd.contains("usepackage") {
                        if !path.ends_with(".sty") { path.push_str(".sty"); }
                    } else {
                        if !path.ends_with(".tex") { path.push_str(".tex"); }
                    }
                    node.children.push(Self::scan_recursive(&path, vfs, visited));
                }
            }
        }

        node
    }
}
