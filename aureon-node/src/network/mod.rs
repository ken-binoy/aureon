use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crate::types::Block;

mod message;
pub use message::*;

/// Represents a connected peer
#[derive(Clone, Debug)]
pub struct Peer {
    pub node_id: String,
    pub version: String,
    pub latest_block_height: u64,
}

/// P2P Network manager for blockchain synchronization
pub struct Network {
    peers: Arc<Mutex<HashMap<String, Peer>>>,
    peer_streams: Arc<Mutex<Vec<TcpStream>>>,
    node_id: String,
    version: String,
}

impl Clone for Network {
    fn clone(&self) -> Self {
        Network {
            peers: Arc::clone(&self.peers),
            peer_streams: Arc::clone(&self.peer_streams),
            node_id: self.node_id.clone(),
            version: self.version.clone(),
        }
    }
}

impl Network {
    /// Create a new network instance
    pub fn new(node_id: String, version: String) -> Self {
        Network {
            peers: Arc::new(Mutex::new(HashMap::new())),
            peer_streams: Arc::new(Mutex::new(Vec::new())),
            node_id,
            version,
        }
    }

    /// Get current node ID
    pub fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    /// Start TCP listener for incoming connections
    pub fn start_listener(&self, address: &str) {
        let listener = match TcpListener::bind(address) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind TCP listener on {}: {}", address, e);
                return;
            }
        };

        let peer_streams = Arc::clone(&self.peer_streams);
        let peers = Arc::clone(&self.peers);

        thread::spawn(move || {
            println!("[Network] Listening on TCP socket");
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    if let Ok(peer_addr) = stream.peer_addr() {
                        println!("[Network] Incoming connection from {}", peer_addr);
                    }
                    
                    peer_streams.lock().unwrap().push(stream.try_clone().unwrap());
                    
                    let peers_clone = Arc::clone(&peers);
                    
                    thread::spawn(move || {
                        if let Ok(stream) = stream.try_clone() {
                            let reader = BufReader::new(stream);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    if let Ok(message) = serde_json::from_str::<Message>(&line) {
                                        println!("[Network] Received {}", message.message_type());
                                        
                                        // Handle PeerInfo updates
                                        if let Message::PeerInfo { 
                                            node_id, version, latest_block_height 
                                        } = message {
                                            let mut peers = peers_clone.lock().unwrap();
                                            peers.insert(node_id.clone(), Peer {
                                                node_id,
                                                version,
                                                latest_block_height,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        });
    }

    /// Connect to a peer
    pub fn add_peer(&self, address: &str, peer_id: Option<String>) {
        let peer_streams = Arc::clone(&self.peer_streams);
        let peers = Arc::clone(&self.peers);
        let address = address.to_string();
        let peer_id = peer_id.unwrap_or_else(|| address.clone());

        thread::spawn(move || {
            match TcpStream::connect(&address) {
                Ok(stream) => {
                    println!("[Network] Connected to peer: {}", address);
                    
                    if let Ok(_) = stream.try_clone() {
                        // Register as placeholder peer (will be updated with PeerInfo)
                        let mut ps = peers.lock().unwrap();
                        ps.insert(peer_id.clone(), Peer {
                            node_id: peer_id,
                            version: "unknown".to_string(),
                            latest_block_height: 0,
                        });
                        drop(ps);
                    }
                    
                    peer_streams.lock().unwrap().push(stream);
                }
                Err(e) => eprintln!("[Network] Failed to connect to {}: {}", address, e),
            }
        });
    }

    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }

    /// Get highest block height from peers
    pub fn get_highest_peer_height(&self) -> u64 {
        self.peers
            .lock()
            .unwrap()
            .values()
            .map(|p| p.latest_block_height)
            .max()
            .unwrap_or(0)
    }

    /// Broadcast message to all peers
    pub fn broadcast(&self, message: &Message) {
        let peer_streams = self.peer_streams.lock().unwrap();
        let data = match serde_json::to_string(&message) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[Network] Failed to serialize message: {}", e);
                return;
            }
        };

        for peer in peer_streams.iter() {
            if let Ok(mut stream) = peer.try_clone() {
                let _ = stream.write_all(data.as_bytes());
                let _ = stream.write_all(b"\n");
                let _ = stream.flush();
            }
        }
    }

    /// Broadcast a block to all peers
    pub fn broadcast_block(&self, block: &Block) {
        let message = Message::Block(block.clone());
        println!("[Network] Broadcasting block");
        self.broadcast(&message);
    }

    /// Request a specific block from peers
    pub fn request_block(&self, height: u64) {
        let message = Message::GetBlock(height);
        println!("[Network] Requesting block #{}", height);
        self.broadcast(&message);
    }

    /// Broadcast peer info to all peers
    pub fn broadcast_peer_info(&self, latest_block_height: u64) {
        let message = Message::PeerInfo {
            node_id: self.node_id.clone(),
            version: self.version.clone(),
            latest_block_height,
        };
        self.broadcast(&message);
    }

    /// Request block range for synchronization
    pub fn request_sync(&self, from_height: u64, to_height: u64) {
        let message = Message::SyncRequest {
            from_height,
            to_height,
        };
        println!("[Network] Requesting sync blocks #{}-#{}", from_height, to_height);
        self.broadcast(&message);
    }

    /// Listen on address (convenience method)
    pub fn listen(&self, address: &str) {
        self.start_listener(address);
        thread::sleep(Duration::from_millis(100)); // Allow listener to start
    }

    /// Handle incoming message (called by network listener)
    /// In a full implementation, this would route to appropriate handlers
    pub fn handle_message(&self, message: Message) -> Result<(), String> {
        match message {
            Message::Ping => {
                self.broadcast(&Message::Pong);
                Ok(())
            }
            Message::Pong => Ok(()), // Just for health checks
            Message::PeerInfo { node_id, version, latest_block_height } => {
                // Update peer info (already done in listener)
                println!("[Network] Peer {} height: {}", node_id, latest_block_height);
                Ok(())
            }
            Message::GetBlock(height) => {
                // In real implementation, would query indexer and respond
                println!("[Network] Peer requesting block #{}", height);
                Ok(())
            }
            Message::GetBlockResponse(block_opt) => {
                // In real implementation, would add to sync queue
                if let Some(block) = block_opt {
                    println!("[Network] Received block response, hash: {}", block.hash);
                } else {
                    println!("[Network] Block not found");
                }
                Ok(())
            }
            Message::Block(block) => {
                println!("[Network] Received block broadcast, hash: {}", block.hash);
                Ok(())
            }
            Message::SyncRequest { from_height, to_height } => {
                println!("[Network] Peer requesting sync blocks #{}-#{}", from_height, to_height);
                Ok(())
            }
            Message::SyncResponse { blocks } => {
                println!("[Network] Received {} blocks for sync", blocks.len());
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let network = Network::new("node1".to_string(), "1.0.0".to_string());
        assert_eq!(network.get_node_id(), "node1");
        assert_eq!(network.peer_count(), 0);
    }

    #[test]
    fn test_peer_height_tracking() {
        let network = Network::new("node1".to_string(), "1.0.0".to_string());
        
        let mut peers = network.peers.lock().unwrap();
        peers.insert("peer1".to_string(), Peer {
            node_id: "peer1".to_string(),
            version: "1.0.0".to_string(),
            latest_block_height: 100,
        });
        peers.insert("peer2".to_string(), Peer {
            node_id: "peer2".to_string(),
            version: "1.0.0".to_string(),
            latest_block_height: 50,
        });
        drop(peers);

        assert_eq!(network.get_highest_peer_height(), 100);
    }

    #[test]
    fn test_message_type_names() {
        assert_eq!(Message::Ping.message_type(), "Ping");
        assert_eq!(Message::Pong.message_type(), "Pong");
        assert_eq!(Message::GetBlock(1).message_type(), "GetBlock");
        assert_eq!(Message::GetBlockResponse(None).message_type(), "GetBlockResponse");
    }
}