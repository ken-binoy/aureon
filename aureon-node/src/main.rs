mod consensus;
mod types;
mod config;
mod wasm;
mod zk;
mod mpt;
mod db;
mod state_processor;
mod simulated_processor;
mod network;

use consensus::get_engine;
use config::load_consensus_type;
use types::Transaction;
use wasm::WasmRuntime;

use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::thread;

use db::Db;
use mpt::MerklePatriciaTrie;
use state_processor::StateProcessor;
use network::Network;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // === Execute Contract Mode (Skip full node setup) ===
    if args.len() > 1 && args[1] == "execute-contract" {
        return run_execute_contract();
    }

    // === Load Consensus Type ===
    let consensus_type = load_consensus_type();
    println!("Selected Consensus: {:?}", consensus_type);

    // === Initialize Consensus Engine ===
    let engine = get_engine(consensus_type);

    // === Initialize Networking ===
    let network = Network::new();
    let network_clone = network.clone();

    // Add peer addresses manually (adjust as needed)
    network.add_peer("127.0.0.1:6001");
    network.add_peer("127.0.0.1:6002");

    thread::spawn(move || {
        network_clone.listen("127.0.0.1:6000"); // Change port per node
    });

    // === Sample Transactions ===
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

    // === Set up Database and Trie ===
    let db = Db::open("aureon_db");
    let mut trie = MerklePatriciaTrie::new();

    // === Initialize Account Balances ===
    let initial_balances: HashMap<&str, u64> = [
        ("Alice", 100),
        ("Charlie", 100),
    ]
    .into();

    for (account, balance) in &initial_balances {
        db.put(account.as_bytes(), &balance.to_le_bytes());
        trie.insert(account.as_bytes().to_vec(), balance.to_le_bytes().to_vec());
    }

    // === Capture Pre-State Root ===
    let pre_state_root = trie.root_hash();

    // === Simulate Transactions for Post-State Root ===
    let sim_processor = StateProcessor::new(&db, &mut trie);
    let post_state_root = sim_processor.simulate_block(&transactions);

    // === Produce and Validate Block ===
    let block = engine.produce_block(
        transactions.clone(),
        pre_state_root.clone(),
        post_state_root.clone(),
    );

    println!("\n--- Produced Block ---\n{:#?}", block);

    let is_valid = engine.validate_block(&block, pre_state_root.clone(), post_state_root.clone());
    println!("Is Block Valid? {}\n", is_valid);

    // === Broadcast the Block ===
    network.broadcast_block(&block);

    // === Commit Block to State ===
    let mut processor = StateProcessor::new(&db, &mut trie);
    let committed_root = processor.apply_block(&block);
    println!("Committed State Root: 0x{}", hex::encode(&committed_root));

    // === WASM Smart Contract Execution ===
    let contracts_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/contracts");
    if Path::new(contracts_dir).exists() {
        println!("\n--- Executing WASM Contracts ---");
        for entry in fs::read_dir(contracts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                println!("Running: {:?}", path);
                let wasm_bytes = fs::read(&path)?;
                let wasm_runtime = WasmRuntime::new(&wasm_bytes)?;
                let result = wasm_runtime.execute_contract(&transactions, 10_000)?;
                println!("Result: {}\n", result);
            }
        }
    } else {
        println!("\nContracts directory '{}' not found. Skipping WASM execution.", contracts_dir);
    }

    // === zk-SNARK Demonstration ===
    println!("\n--- zk-SNARK Proof Demo ---");
    zk::generate_and_verify_proof(3, 5)?;

    // === Final Account Balances ===
    println!("\n--- Final Account Balances ---");
    for account in ["Alice", "Bob", "Charlie", "Dave"] {
        let balance = processor.get_balance(account);
        println!("{}: {}", account, balance);
    }

    Ok(())
}

fn run_execute_contract() -> anyhow::Result<()> {
    use std::env;
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("Usage: execute-contract <contract_hash> <caller> <gas_limit> --input_args <arg1,arg2,...>");
        std::process::exit(1);
    }

    let contract_hash = &args[2];
    let caller = &args[3];
    let gas_limit: u64 = args[4].parse()?;
    let input_args = if args.len() > 6 && args[5] == "--input_args" {
        args[6].split(',').map(|s| s.to_string()).collect::<Vec<_>>()
    } else {
        vec![]
    };

    println!("Executing contract {} as {} with gas {} and args {:?}", contract_hash, caller, gas_limit, input_args);

    // Placeholder for actual contract execution logic
    // You can reuse WasmRuntime or your internal execution pipeline here

    Ok(())
}
