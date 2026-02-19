use tokio::fs;
use std::path::Path;

pub struct IoHandler;

impl IoHandler {
    pub async fn auto_save(content: String, path: &str) -> Result<(), std::io::Error> {
        fs::write(path, content).await
    }
}
