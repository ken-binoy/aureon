use sha3::{Digest, Keccak256};
use serde::{Serialize, Deserialize};
use bincode::{Encode, encode_to_vec, config::standard};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode)]
pub enum Node {
    Branch([Option<Box<Node>>; 16], Option<Vec<u8>>),
    Leaf(Vec<u8>, Vec<u8>),
    Extension(Vec<u8>, Box<Node>),
}

impl Node {
    pub fn hash(&self) -> Vec<u8> {
        let encoded = encode_to_vec(self, standard()).unwrap();
        Keccak256::digest(&encoded).to_vec()
    }
}