use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use shared::types::block::Block;
use std::{process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::broadcast,
    task::JoinHandle,
};
use tokio_stream::StreamExt;

pub struct AnvilProcess {
    pub name: String,
    pub chain_id: u64,
    pub port: u16,
    pub block_time: u64,
    child: Option<Child>,
    pub log_handles: Vec<JoinHandle<()>>,
    pub log_tx: Arc<broadcast::Sender<String>>,
    pub block_tx: Arc<broadcast::Sender<Block>>,
    pub block_handle: Option<JoinHandle<()>>,
}

impl AnvilProcess {
    pub fn new(
        name: String,
        chain_id: u64,
        port: u16,
        block_time: u64,
        log_tx: Arc<broadcast::Sender<String>>,
        block_tx: Arc<broadcast::Sender<Block>>,
    ) -> Self {
        Self {
            name,
            chain_id,
            port,
            block_time,
            child: None,
            log_handles: Vec::new(),
            log_tx,
            block_tx,
            block_handle: None,
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

        let port = self.port;
        let block_tx = self.block_tx.clone();
        let block_handle = tokio::spawn(async move {
            if let Err(e) = async {
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                let ws = WsConnect::new(format!("ws://127.0.0.1:{}", port));
                let provider = ProviderBuilder::new().connect_ws(ws).await?;
                let mut stream = provider.subscribe_blocks().await?.into_stream();

                while let Some(header) = stream.next().await {
                    let _ = block_tx.send(Block {
                        number: header.number,
                        hash: header.hash.to_string(),
                        time: header.timestamp,
                    });
                }
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                println!("Block stream error: {:?}", e);
            }
        });
        self.block_handle = Some(block_handle);

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
