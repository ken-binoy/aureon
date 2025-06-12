mod consensus;
mod types;
mod config;
mod wasm;
mod zk;
mod mpt;
mod db;
mod state_processor;

use consensus::get_engine;
use config::load_consensus_type;
use types::Transaction;
use wasm::WasmRuntime;
use std::{fs, path::Path, collections::HashMap};

use db::Db;
use mpt::MerklePatriciaTrie;
use state_processor::StateProcessor;

fn main() -> anyhow::Result<()> {
    // Load consensus type from config.json
    let consensus_type = load_consensus_type();
    println!("Selected Consensus: {:?}", consensus_type);

    // Initialize consensus engine
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

    // Produce and validate block
    let block = engine.produce_block(transactions.clone());
    println!("Produced Block:\n{:#?}", block);

    let is_valid = engine.validate_block(&block);
    println!("Is Block Valid? {}\n", is_valid);

    // Execute WASM contracts if present
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

    // Demonstrate zk-SNARK
    println!("Demonstrating Zero-Knowledge Proof:");
    zk::generate_and_verify_proof(3, 5)?;

    // === Apply Block Transactions to State ===
    let mut db = Db::open("aureon_db");
    let mut trie = MerklePatriciaTrie::new();

    // Set initial balances
    let initial_balances: HashMap<&str, u64> = [
        ("Alice", 100),
        ("Charlie", 100),
    ]
    .into();

    for (account, balance) in &initial_balances {
        db.put(account.as_bytes(), &balance.to_le_bytes());
        trie.insert(account.as_bytes().to_vec(), balance.to_le_bytes().to_vec());
    }

    // Apply block transactions
    let mut processor = StateProcessor::new(&mut db, &mut trie);
    processor.apply_block(&block);

    // Compute new state root
    let new_root = trie.root_hash();
    println!("New State Root: 0x{}", hex::encode(new_root));

    // Print resulting balances
    println!("\nFinal Account Balances:");
    for account in ["Alice", "Bob", "Charlie", "Dave"] {
        let balance = db.get(account.as_bytes())
            .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap_or_default()))
            .unwrap_or(0);
        println!("{}: {}", account, balance);
    }

    Ok(())
}