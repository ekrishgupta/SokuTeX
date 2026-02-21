use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use std::path::PathBuf;
use tokio::sync::mpsc;
use log::info;

pub enum LatexmkEvent {
    BuildStarted,
    BuildFinished(bool), // true if success
}

pub struct LatexmkPvc {
    child: Child,
    stdin: tokio::process::ChildStdin,
}

impl LatexmkPvc {
    pub fn spawn(main_file: PathBuf, event_tx: mpsc::Sender<LatexmkEvent>) -> Result<Self, std::io::Error> {
        let mut child = Command::new("latexmk")
            .arg("-pvc")
            .arg("-pdf")
            .arg("-interaction=nonstopmode")
            // Use -view=none to prevent latexmk from opening a PDF viewer
            .arg("-view=none")
            .arg(main_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().expect("failed to open stdin");
        let stdout = child.stdout.take().expect("failed to open stdout");
        
        // Spawn a task to monitor stdout
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                info!("latexmk: {}", line);
                if line.contains("Latexmk: All targets") && line.contains("are up-to-date") {
                    let _ = event_tx.send(LatexmkEvent::BuildFinished(true)).await;
                } else if line.contains("Latexmk: Run number") {
                    let _ = event_tx.send(LatexmkEvent::BuildStarted).await;
                } else if line.contains("Errors during processing") {
                    let _ = event_tx.send(LatexmkEvent::BuildFinished(false)).await;
                }
            }
        });

        Ok(Self { child, stdin })
    }

    pub async fn trigger_rebuild(&mut self) -> tokio::io::Result<()> {
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn kill(mut self) -> tokio::io::Result<()> {
        self.child.kill().await
    }
}
