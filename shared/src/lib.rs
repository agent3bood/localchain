use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: u64,
    pub port: u16,
    pub block_time: u64,
    pub status: ChainStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainStatus {
    Stopped,
    Running,
    Starting,
    Error,
}
