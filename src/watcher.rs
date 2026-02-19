use notify::{Watcher, RecursiveMode, Config};
use std::path::Path;
use tokio::sync::mpsc;

pub enum FileEvent {
    Modified(String),
}

pub struct FileWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(tx: mpsc::Sender<FileEvent>) -> notify::Result<Self> {
        let (sync_tx, sync_rx) = std::sync::mpsc::channel();

        let watcher = notify::RecommendedWatcher::new(sync_tx, Config::default())?;
        
        // Spawn a bridge thread from sync mpsc to tokio mpsc
        std::thread::spawn(move || {
            while let Ok(res) = sync_rx.recv() {
                match res {
                    Ok(event) => {
                        if event.kind.is_modify() {
                            for path in event.paths {
                                if let Some(path_str) = path.to_str() {
                                    let _ = tx.blocking_send(FileEvent::Modified(path_str.to_string()));
                                }
                            }
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }

    pub fn watch(&mut self, path: &str) -> notify::Result<()> {
        self._watcher.watch(Path::new(path), RecursiveMode::Recursive)
    }
}
