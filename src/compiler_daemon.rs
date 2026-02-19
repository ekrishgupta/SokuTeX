use tokio::sync::mpsc;
use crate::compiler::Compiler;

pub enum CompileRequest {
    Compile {
        latex: String,
        response: tokio::sync::oneshot::Sender<Vec<u8>>,
    },
}

pub struct CompilerDaemon {
    receiver: mpsc::Receiver<CompileRequest>,
}

impl CompilerDaemon {
    pub fn new(receiver: mpsc::Receiver<CompileRequest>) -> Self {
        Self { receiver }
    }

    pub async fn run(mut self) {
        let compiler = Compiler::new();
        while let Some(request) = self.receiver.recv().await {
            match request {
                CompileRequest::Compile { latex, response } => {
                    // Simulate compilation for now
                    if let Ok(pdf) = Compiler::compile(&latex) {
                        let _ = response.send(pdf);
                    }
                }
            }
        }
    }
}
