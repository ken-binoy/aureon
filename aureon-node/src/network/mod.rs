use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::types::Block;

mod message;
pub use message::*;

pub struct Network {
    peers: Arc<Mutex<Vec<TcpStream>>>,
}

impl Clone for Network {
    fn clone(&self) -> Self {
        Network {
            peers: Arc::clone(&self.peers),
        }
    }
}

impl Network {
    pub fn new() -> Self {
        Network {
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_listener(&self, address: &str) {
        let listener = TcpListener::bind(address).expect("Failed to bind TCP listener");
        let peers = Arc::clone(&self.peers);

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    println!("Incoming connection from {:?}", stream.peer_addr());
                    peers.lock().unwrap().push(stream.try_clone().unwrap());

                    let _peers_clone = Arc::clone(&peers);
                    thread::spawn(move || {
                        let reader = BufReader::new(stream);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                if let Ok(message) = serde_json::from_str::<Message>(&line) {
                                    println!("Received: {:?}", message);
                                    match message {
                                        Message::Ping => println!("Received Ping"),
                                        Message::Pong => println!("Received Pong"),
                                        Message::Transactions(txns) => {
                                            println!("Received transactions: {:?}", txns);
                                        }
                                        Message::Block(block) => {
                                            println!("Received block: {:?}", block);
                                        }
                                        Message::NewBlock(raw) => {
                                            println!("Received raw block string: {}", raw);
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

    pub fn add_peer(&self, address: &str) {
        if let Ok(stream) = TcpStream::connect(address) {
            println!("Connected to peer: {}", address);
            self.peers.lock().unwrap().push(stream);
        } else {
            println!("Failed to connect to peer: {}", address);
        }
    }

    pub fn broadcast(&self, message: &Message) {
        let peers = self.peers.lock().unwrap();
        let data = serde_json::to_string(&message).unwrap();

        for peer in peers.iter() {
            let _ = peer.try_clone().and_then(|mut stream| {
                stream.write_all(data.as_bytes())?;
                stream.write_all(b"\n")?;
                stream.flush()
            });
        }
    }

    pub fn broadcast_block(&self, block: &Block) {
        let message = Message::Block(block.clone());
        self.broadcast(&message);
    }

    pub fn listen(&self, address: &str) {
        println!("Listening on {}", address);
        self.start_listener(address);
    }
}