use regex::Regex;
use std::collections::HashSet;
use crate::vfs::Vfs;

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub children: Vec<DependencyNode>,
}

pub struct DependencyScanner;

impl DependencyScanner {
    pub fn scan(root_file: &str, vfs: &Vfs) -> DependencyNode {
        let mut visited = HashSet::new();
        Self::scan_recursive(root_file, vfs, &mut visited)
    }

    fn scan_recursive(file_name: &str, vfs: &Vfs, visited: &mut HashSet<String>) -> DependencyNode {
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
            let re = Regex::new(r"\\(?:input|include)\{([^}]*)\}").unwrap();
            
            for cap in re.captures_iter(&content) {
                let mut sub_file = cap[1].to_string();
                if !sub_file.ends_with(".tex") {
                    sub_file.push_str(".tex");
                }
                node.children.push(Self::scan_recursive(&sub_file, vfs, visited));
            }
        }

        node
    }
}
