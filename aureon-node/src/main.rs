mod consensus;
mod types;
mod config;
mod wasm;

use consensus::get_engine;
use config::load_consensus_type;
use types::Transaction;
use wasm::WasmRuntime;
use std::fs;

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
    let contracts_dir = "src/contracts";
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

    Ok(())
}
