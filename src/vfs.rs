use dashmap::DashMap;
use std::sync::Arc;
use ahash::RandomState;
use std::path::Path;
use log::info;

pub struct Vfs {
    files: Arc<DashMap<String, Vec<u8>, RandomState>>,
    pub root_dir: Option<String>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: Arc::new(DashMap::with_hasher(RandomState::new())),
            root_dir: None,
        }
    }

    pub fn write_file(&self, path: &str, content: Vec<u8>) {
        self.files.insert(path.to_string(), content);
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        // Try VFS first
        if let Some(v) = self.files.get(path) {
            return Some(v.value().clone());
        }
        // Fall back to disk if we have a root dir
        if let Some(ref root) = self.root_dir {
            let disk_path = Path::new(root).join(path);
            if let Ok(data) = std::fs::read(&disk_path) {
                // Cache it in VFS for next time
                self.files.insert(path.to_string(), data.clone());
                return Some(data);
            }
        }
        None
    }

    pub fn get_all_files(&self) -> Arc<DashMap<String, Vec<u8>, RandomState>> {
        self.files.clone()
    }

    /// Load all .tex, .bib, .sty, .cls files from a directory into the VFS
    pub fn load_directory(&mut self, dir: &str) {
        self.root_dir = Some(dir.to_string());
        let path = Path::new(dir);
        if path.is_dir() {
            self.load_dir_recursive(path, path);
        } else if path.is_file() {
            // If a single file is given, load it and set root to its parent
            if let Some(parent) = path.parent() {
                self.root_dir = Some(parent.to_string_lossy().to_string());
                if let Ok(data) = std::fs::read(path) {
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    info!("VFS: loaded {}", name);
                    self.files.insert(name, data);
                }
                // Also load sibling files
                self.load_dir_recursive(parent, parent);
            }
        }
    }

    fn load_dir_recursive(&self, dir: &Path, root: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden dirs and target/
                    let name = path.file_name().unwrap().to_string_lossy();
                    if !name.starts_with('.') && name != "target" {
                        self.load_dir_recursive(&path, root);
                    }
                } else if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if matches!(ext, "tex" | "bib" | "sty" | "cls" | "png" | "jpg" | "jpeg" | "pdf") {
                        if let Ok(data) = std::fs::read(&path) {
                            let relative = path.strip_prefix(root)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();
                            info!("VFS: loaded {} ({} bytes)", relative, data.len());
                            self.files.insert(relative, data);
                        }
                    }
                }
            }
        }
    }
}
