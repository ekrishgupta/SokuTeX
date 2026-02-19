use tokio::sync::{mpsc, oneshot};
use crate::latexmk::{LatexmkPvc, LatexmkEvent};
use crate::compiler::Compiler;
use crate::vfs::Vfs;
use crate::config::CompileBackend;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use log::info;

pub enum CompileRequest {
    Compile {
        latex: String,
        backend: CompileBackend,
        draft: bool,
        response: oneshot::Sender<Vec<u8>>,
    },
}

pub struct CompilerDaemon {
    receiver: mpsc::Receiver<CompileRequest>,
    latexmk: LatexmkPvc,
    event_rx: mpsc::Receiver<LatexmkEvent>,
    pending_response: Option<oneshot::Sender<Vec<u8>>>,
    compiler: Compiler,
    vfs: Arc<Vfs>,
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
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(request) = self.receiver.recv() => {
                    match request {
                        CompileRequest::Compile { latex, backend, draft, response } => {
                            self.compiler.set_backend(backend);

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
                                    let _ = response.send(pdf);
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
                                        let _ = response.send(pdf_data);
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
