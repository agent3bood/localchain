use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Nice name for UI display only
    pub name: String,
    /// Chain Identifier, must be unique, used in API
    pub id: u64,
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

impl ChainConfig {
    pub fn next(existing: &Vec<ChainConfig>) -> ChainConfig {
        ChainConfig {
            name: format!("Chain-{}", existing.len() + 1),
            id: existing.iter().map(|c| c.id).max().unwrap_or(0) + 1,
            port: existing.iter().map(|c| c.port).max().unwrap_or(8544) + 1,
            block_time: 1,
            status: ChainStatus::Stopped,
        }
    }
}
