mod consensus;
mod types;
mod config;

use consensus::get_engine;
use config::load_consensus_type;
use types::Transaction;

fn main() {
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
}