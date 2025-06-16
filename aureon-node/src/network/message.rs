use crate::types::Block;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableTransaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    NewBlock(String), // Optional: raw string of block JSON
    Block(Block),     // Structured Block
    Transactions(Vec<SerializableTransaction>),
    Ping,
    Pong,
}