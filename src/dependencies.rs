use regex::Regex;
use dashmap::DashSet;
use rayon::prelude::*;
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
        let visited = DashSet::new();
        Self::scan_recursive(root_file, vfs, &visited)
    }

    fn scan_recursive(file_name: &str, vfs: &Vfs, visited: &DashSet<String>) -> DependencyNode {
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
            
            let mut matches = Vec::new();
            for cap in re.captures_iter(&content) {
                let matched_cmd = cap.get(0).unwrap().as_str().to_string();
                let sub_files = cap[1].to_string();
                matches.push((matched_cmd, sub_files));
            }

            node.children = matches.into_par_iter()
                .flat_map(|(matched_cmd, sub_files)| {
                    let mut parts = Vec::new();
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
                        parts.push(path);
                    }
                    parts
                })
                .map(|path| Self::scan_recursive(&path, vfs, visited))
                .collect();
        }

        node
    }
}
