use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Block {
    pub beneficiary: String,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub number: u64,
    pub hash: String,
    pub time: u64,
    pub nonce: String,
    pub transactions: u64,
}

impl Block {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
