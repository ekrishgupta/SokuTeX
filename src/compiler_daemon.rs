use tokio::sync::{mpsc, oneshot};
use crate::latexmk::{LatexmkPvc, LatexmkEvent};
use crate::compiler::Compiler;
use crate::vfs::Vfs;
use crate::config::CompileBackend;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use log::info;

pub struct CompileResult {
    pub pdf: Vec<u8>,
    pub revision: u64,
}

pub enum CompileRequest {
    Compile {
        latex: String,
        backend: CompileBackend,
        draft: bool,
        active_file: Option<String>,
        response: oneshot::Sender<CompileResult>,
    },
}

pub struct CompilerDaemon {
    receiver: mpsc::Receiver<CompileRequest>,
    latexmk: LatexmkPvc,
    event_rx: mpsc::Receiver<LatexmkEvent>,
    pending_response: Option<oneshot::Sender<CompileResult>>,
    compiler: Compiler,
    vfs: Arc<Vfs>,
    revision: u64,
}

impl CompilerDaemon {
    pub fn new(receiver: mpsc::Receiver<CompileRequest>, vfs: Arc<Vfs>) -> Self {
        let (event_tx, event_rx) = mpsc::channel(10);
        let latexmk = LatexmkPvc::spawn(PathBuf::from("main.tex"), event_tx).expect("Failed to spawn latexmk");
        Self { 
            receiver, 
            latexmk, 
            event_rx,
            pending_response: None,
            compiler: Compiler::new(),
            vfs,
            revision: 0,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(request) = self.receiver.recv() => {
                    match request {
                        CompileRequest::Compile { latex, backend, draft, active_file, response } => {
                            self.compiler.set_backend(backend);
                            self.compiler.active_file = active_file;

                            if backend == CompileBackend::Latexmk {
                                // Incremental Optimization: Inject \includeonly if possible
                                let (optimized_latex, is_incremental) = self.compiler.optimize_latex(&latex, draft, &self.vfs);
                                
                                if is_incremental {
                                    info!("Transparent Incremental Compilation: injecting \\includeonly");
                                }

                                // 1. Save optimized content to main.tex
                                if let Ok(_) = fs::write("main.tex", optimized_latex).await {
                                    // 2. Trigger rebuild via persistent handle
                                    let _ = self.latexmk.trigger_rebuild().await;
                                    // 3. Store response channel to send back PDF when build finishes
                                    self.pending_response = Some(response);
                                }
                            } else {
                                // Use Internal or Tectonic
                                if let Ok(pdf) = self.compiler.compile(&latex, draft, &self.vfs) {
                                    self.revision += 1;
                                    let _ = response.send(CompileResult { pdf, revision: self.revision });
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
                                        self.revision += 1;
                                        let _ = response.send(CompileResult { pdf: pdf_data, revision: self.revision });
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
}
