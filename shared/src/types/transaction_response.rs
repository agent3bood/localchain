use crate::types::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionResponse {
    pub transaction: Transaction,
}
