use tokio::sync::{mpsc, oneshot};
use crate::latexmk::{LatexmkPvc, LatexmkEvent};
use crate::compiler::Compiler;
use crate::vfs::Vfs;
use crate::config::CompileBackend;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use log::{info, error};
use std::hash::Hash;

pub struct CompileResult {
    pub pdf: Vec<u8>,
    pub revision: u64,
    #[allow(dead_code)]
    pub synctex_data: Option<Vec<u8>>,
}

pub enum CompileRequest {
    Compile {
        latex: String,
        backend: CompileBackend,
        draft: bool,
        focus_mode: bool,
        active_file: Option<String>,
        response: oneshot::Sender<CompileResult>,
    },
}

pub struct CompilerDaemon {
    receiver: mpsc::Receiver<CompileRequest>,
    latexmk: Option<LatexmkPvc>,
    event_rx: mpsc::Receiver<LatexmkEvent>,
    pending_response: Option<oneshot::Sender<CompileResult>>,
    compiler: Compiler,
    vfs: Arc<Vfs>,
    revision: u64,
    last_pdf_hash: u64,
}

impl CompilerDaemon {
    pub fn new(receiver: mpsc::Receiver<CompileRequest>, vfs: Arc<Vfs>) -> Self {
        let (event_tx, event_rx) = mpsc::channel(10);
        
        let latexmk = match LatexmkPvc::spawn(PathBuf::from("main.tex"), event_tx) {
            Ok(pvc) => Some(pvc),
            Err(e) => {
                error!("Failed to spawn latexmk: {}. Latexmk backend will be unavailable.", e);
                None
            }
        };

        Self { 
            receiver, 
            latexmk, 
            event_rx,
            pending_response: None,
            compiler: Compiler::new(),
            vfs,
            revision: 0,
            last_pdf_hash: 0,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(request) = self.receiver.recv() => {
                    match request {
                        CompileRequest::Compile { latex, mut backend, draft, focus_mode, active_file, response } => {
                            if draft {
                                // Force Internal for near-instant feedback in draft mode
                                backend = CompileBackend::Internal;
                            }
                            self.compiler.set_backend(backend);
                            self.compiler.active_file = active_file;

                            if backend == CompileBackend::Latexmk {
                                if let Some(ref mut latexmk) = self.latexmk {
                                    // Incremental Optimization: Inject \includeonly if possible
                                    let (optimized_latex, is_incremental, deltas) = self.compiler.optimize_latex(&latex, draft, focus_mode, &self.vfs);
                                    
                                    for delta in &deltas {
                                        info!("Delta detected for Latexmk: {} ({} -> {})", delta.path, delta.old_hash, delta.new_hash);
                                    }

                                    if is_incremental {
                                        info!("Transparent Incremental Compilation: injecting \\includeonly");
                                    }

                                    // 1. Save optimized content to main.tex
                                    if let Ok(_) = fs::write("main.tex", optimized_latex).await {
                                        // 2. Trigger rebuild via persistent handle
                                        let _ = latexmk.trigger_rebuild().await;
                                        // 3. Store response channel to send back PDF when build finishes
                                        self.pending_response = Some(response);
                                    }
                                } else {
                                    // Fallback if latexmk failed to start
                                    error!("Latexmk requested but not available. Falling back to internal.");
                                    if let Ok(pdf) = self.compiler.compile(&latex, draft, focus_mode, &self.vfs) {
                                        self.update_revision_and_send(pdf, None, response);
                                    }
                                }
                            } else {
                                // Use Internal or Tectonic
                                if let Ok(pdf) = self.compiler.compile(&latex, draft, focus_mode, &self.vfs) {
                                    self.update_revision_and_send(pdf, None, response);
                                }
                            }
                        }
                    }
                }
                Some(event) = self.event_rx.recv() => {
                    match event {
                        LatexmkEvent::BuildFinished(success) => {
                            if success {
                                if let Some(response) = self.pending_response.take() {
                                    // 4. Read generated PDF and send back
                                    if let Ok(pdf_data) = fs::read("main.pdf").await {
                                        let synctex_data = if let Ok(data) = fs::read("main.synctex.gz").await {
                                            Some(data)
                                        } else if let Ok(data) = fs::read("main.synctex").await {
                                            Some(data)
                                        } else {
                                            None
                                        };
                                        self.update_revision_and_send(pdf_data, synctex_data, response);
                                    }
                                }
                            }
                        }
                        LatexmkEvent::BuildStarted => {
                            // Could notify UI that build is in progress
                        }
                    }
                }
            }
        }
    }

    fn update_revision_and_send(&mut self, pdf: Vec<u8>, synctex: Option<Vec<u8>>, response: oneshot::Sender<CompileResult>) {
        let mut hasher = ahash::AHasher::default();
        use std::hash::Hasher;
        pdf.hash(&mut hasher);
        let hash = hasher.finish();

        if hash != self.last_pdf_hash {
            self.revision += 1;
            self.last_pdf_hash = hash;
        }

        let _ = response.send(CompileResult { 
            pdf, 
            revision: self.revision,
            synctex_data: synctex,
        });
    }
}
