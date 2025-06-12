use super::node::Node;
use super::util::nibble_key;

pub struct MerklePatriciaTrie {
    root: Option<Node>,
}

impl MerklePatriciaTrie {
    pub fn new() -> Self {
        MerklePatriciaTrie { root: None }
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let _nibbles = nibble_key(&key);
        self.root = Some(Node::Leaf(key, value));
    }

    pub fn get(&self, key: Vec<u8>) -> Option<&[u8]> {
        match &self.root {
            Some(Node::Leaf(k, v)) if *k == key => Some(v),
            _ => None,
        }
    }

    pub fn root_hash(&self) -> Vec<u8> {
        match &self.root {
            Some(node) => node.hash(),
            None => vec![],
        }
    }
}