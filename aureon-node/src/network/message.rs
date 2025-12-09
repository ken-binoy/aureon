use crate::types::Block;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableTransaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

/// P2P Network Messages for block synchronization and consensus
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    // Health checks
    Ping,
    Pong,
    
    // Block synchronization
    Block(Block),                          // Single block broadcast
    NewBlock(String),                      // Block announcement (raw JSON)
    GetBlock(u64),                         // Request block by height
    GetBlockResponse(Option<Block>),       // Response to GetBlock
    
    // State synchronization
    SyncRequest {
        from_height: u64,
        to_height: u64,
    },
    SyncResponse {
        blocks: Vec<Block>,
    },
    
    // Peer info
    PeerInfo {
        node_id: String,
        version: String,
        latest_block_height: u64,
    },
    
    // Legacy transaction support
    Transactions(Vec<SerializableTransaction>),
}

impl Message {
    /// Get message type name for logging
    pub fn message_type(&self) -> &str {
        match self {
            Message::Ping => "Ping",
            Message::Pong => "Pong",
            Message::Block(_) => "Block",
            Message::NewBlock(_) => "NewBlock",
            Message::GetBlock(_) => "GetBlock",
            Message::GetBlockResponse(_) => "GetBlockResponse",
            Message::SyncRequest { .. } => "SyncRequest",
            Message::SyncResponse { .. } => "SyncResponse",
            Message::PeerInfo { .. } => "PeerInfo",
            Message::Transactions(_) => "Transactions",
        }
    }
}