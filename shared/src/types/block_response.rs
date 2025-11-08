use crate::types::block::Block;
use crate::types::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockResponse {
    pub block: Block,
    pub transactions: Vec<Transaction>,
}
