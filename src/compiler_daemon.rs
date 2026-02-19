use tokio::sync::{mpsc, oneshot};
use crate::latexmk::{LatexmkPvc, LatexmkEvent};
use crate::compiler::Compiler;
use crate::config::CompileBackend;
use std::path::PathBuf;
use tokio::fs;

pub enum CompileRequest {
    Compile {
        latex: String,
        backend: CompileBackend,
        response: oneshot::Sender<Vec<u8>>,
    },
}

pub struct CompilerDaemon {
    receiver: mpsc::Receiver<CompileRequest>,
    latexmk: Option<LatexmkPvc>,
    event_rx: mpsc::Receiver<LatexmkEvent>,
    event_tx: mpsc::Sender<LatexmkEvent>,
    pending_response: Option<oneshot::Sender<Vec<u8>>>,
    compiler: Compiler,
}

impl CompilerDaemon {
    pub fn new(receiver: mpsc::Receiver<CompileRequest>) -> Self {
        let (event_tx, event_rx) = mpsc::channel(10);
        Self { 
            receiver, 
            latexmk: None, 
            event_rx,
            event_tx,
            pending_response: None,
            compiler: Compiler::new(CompileBackend::Internal),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(request) = self.receiver.recv() => {
                    match request {
                        CompileRequest::Compile { latex, backend, response } => {
                            match backend {
                                CompileBackend::Latexmk => {
                                    // Ensure latexmk is started
                                    if self.latexmk.is_none() {
                                        if let Ok(lmk) = LatexmkPvc::spawn(PathBuf::from("main.tex"), self.event_tx.clone()) {
                                            self.latexmk = Some(lmk);
                                        }
                                    }

                                    if let Some(lmk) = &mut self.latexmk {
                                        // 1. Save content to main.tex
                                        if let Ok(_) = fs::write("main.tex", latex).await {
                                            // 2. Trigger rebuild via persistent handle
                                            let _ = lmk.trigger_rebuild().await;
                                            // 3. Store response channel to send back PDF when build finishes
                                            self.pending_response = Some(response);
                                        }
                                    }
                                }
                                _ => {
                                    // Internal or Tectonic
                                    self.compiler.set_backend(backend);
                                    // In real impl, use active VFS
                                    let vfs = crate::vfs::Vfs::new();
                                    if let Ok(pdf) = self.compiler.compile(&latex, &vfs) {
                                        let _ = response.send(pdf);
                                    }
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
