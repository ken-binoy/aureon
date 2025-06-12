mod consensus;
mod types;
mod config;
mod wasm;
mod zk;
mod mpt;

use mpt::MerklePatriciaTrie;

use consensus::get_engine;
use config::load_consensus_type;
use types::Transaction;
use wasm::WasmRuntime;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Load consensus type from config.json (e.g., PoW or PoS)
    let consensus_type = load_consensus_type();
    println!("Selected Consensus: {:?}", consensus_type);

    // Initialize the appropriate consensus engine
    let engine = get_engine(consensus_type);

    // Simulate sample transactions
    let transactions = vec![
        Transaction {
            from: "Alice".into(),
            to: "Bob".into(),
            amount: 50,
        },
        Transaction {
            from: "Charlie".into(),
            to: "Dave".into(),
            amount: 75,
        },
    ];

    // Produce a new block using the selected consensus engine
    let block = engine.produce_block(transactions.clone());
    println!("Produced Block:\n{:#?}", block);

    // Validate the produced block
    let is_valid = engine.validate_block(&block);
    
    println!("Is Block Valid? {}\n", is_valid);
    
    // Check if contracts directory exists
    let contracts_dir = "src/contracts";
    if Path::new(contracts_dir).exists() {
        println!("Loading WASM contracts from: {}", contracts_dir);
        for entry in fs::read_dir(contracts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                println!("Loading WASM from: {:?}", path);
                let wasm_bytes = fs::read(&path)?;
                let wasm_runtime = WasmRuntime::new(&wasm_bytes)?;
                let result = wasm_runtime.execute_contract(&transactions, 10_000)?;
                println!("WASM Execution Result: {}\n", result);
            }
        }
    } else {
        println!("Contracts directory '{}' not found, skipping WASM execution.\n", contracts_dir);
    }

    // Demonstrate zero-knowledge proof
    println!("Demonstrating Zero-Knowledge Proof:");
    zk::generate_and_verify_proof(3, 5)?;

    let mut trie = MerklePatriciaTrie::new();
    trie.insert(b"foo".to_vec(), b"bar".to_vec());
    trie.insert(b"fool".to_vec(), b"baz".to_vec());
    let root_hash = trie.root_hash();
    println!("Trie Root Hash: 0x{}", hex::encode(root_hash));

    test_trie_demo();
    
    Ok(())
}

fn test_trie_demo() {
    println!("\nMerkle Patricia Trie Demo:");

    let mut trie = MerklePatriciaTrie::new();
    trie.insert(b"foo".to_vec(), b"bar".to_vec());
    trie.insert(b"fool".to_vec(), b"baz".to_vec());

    let val = trie.get(b"foo".to_vec());
    println!("Get 'foo': {:?}", val.map(|v| String::from_utf8_lossy(&v)));

    let root = trie.root_hash();
    println!("Merkle Patricia Trie Root Hash: 0x{}", hex::encode(root));
}