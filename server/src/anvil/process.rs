use std::{process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::broadcast,
    task::JoinHandle,
};

pub struct AnvilProcess {
    pub name: String,
    pub chain_id: u64,
    pub port: u16,
    pub block_time: u64,
    child: Option<Child>,
    pub log_handles: Vec<JoinHandle<()>>,
    pub log_tx: Arc<broadcast::Sender<String>>,
}

impl AnvilProcess {
    pub fn new(
        name: String,
        chain_id: u64,
        port: u16,
        block_time: u64,
        log_tx: Arc<broadcast::Sender<String>>,
    ) -> Self {
        Self {
            name,
            chain_id,
            port,
            block_time,
            child: None,
            log_handles: Vec::new(),
            log_tx,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if self.child.is_some() {
            self.stop().await?;
        }
        let mut cmd = Command::new("anvil");
        cmd.arg("--port")
            .arg(self.port.to_string())
            .arg("--chain-id")
            .arg(self.chain_id.to_string())
            .arg("--block-time")
            .arg(self.block_time.to_string());

        println!(
            "[{}] Starting Anvil (chainId={}, port={}, blockTime={:?})",
            self.name, self.chain_id, self.port, self.block_time
        );
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        let log_tx = self.log_tx.clone();

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let handle = tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = log_tx.send(format!("[stdout] {}", line));
                }
            });
            self.log_handles.push(handle);
        }

        let log_tx = self.log_tx.clone();
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            let handle = tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = log_tx.send(format!("[stderr] {}", line));
                }
            });
            self.log_handles.push(handle);
        }
        self.child = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            match child.kill().await {
                Ok(_) => {
                    let _ = child.wait();
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        self.child = None;
        Ok(())
    }

    pub async fn restart(&mut self) -> Result<(), String> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }
}
