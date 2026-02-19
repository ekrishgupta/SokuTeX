use dashmap::DashMap;
use std::sync::Arc;

pub struct Vfs {
    files: Arc<DashMap<String, Vec<u8>>>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: Arc::new(DashMap::new()),
        }
    }

    pub fn write_file(&self, path: &str, content: Vec<u8>) {
        self.files.insert(path.to_string(), content);
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.get(path).map(|v| v.value().clone())
    }
}
