use dashmap::DashMap;
use std::sync::Arc;
use ahash::RandomState;

pub struct Vfs {
    files: Arc<DashMap<String, Vec<u8>, RandomState>>,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    pub fn write_file(&self, path: &str, content: Vec<u8>) {
        self.files.insert(path.to_string(), content);
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.get(path).map(|v| v.value().clone())
    }

    pub fn get_all_files(&self) -> Arc<DashMap<String, Vec<u8>, RandomState>> {
        self.files.clone()
    }
}
