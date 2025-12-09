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
mod contract_registry;
mod api;
mod indexer;
mod mempool;
mod block_producer;
mod crypto;
mod sync;
mod multinode_test;
mod metrics;
mod logging;
mod monitoring;
mod metrics_tracker;
mod shard_coordinator;
mod shard_manager;
mod cross_shard_protocol;
mod shard_sync;

use consensus::get_engine;
use config::AureonConfig;
use types::Transaction;
use wasm::WasmRuntime;

use std::fs;
use std::path::Path;
use std::thread;
use std::sync::{Arc, Mutex};

use db::Db;
use mpt::MerklePatriciaTrie;
use state_processor::StateProcessor;
use network::Network;
use contract_registry::ContractRegistry;
use api::start_api_server;
use indexer::BlockchainIndexer;
use mempool::TransactionMempool;
use metrics::Metrics;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // === Generate Keypair Mode ===
    if args.len() > 1 && args[1] == "keygen" {
        let (secret, public) = crypto::generate_keypair();
        println!("Generated Ed25519 keypair:");
        println!("Secret Key: {}", secret);
        println!("Public Key: {}", public);
        println!("\nStore the secret key safely. Use it to sign transactions.");
        return Ok(());
    }

    // === Execute Contract Mode (Skip full node setup) ===
    if args.len() > 1 && args[1] == "execute-contract" {
        return run_execute_contract();
    }

    // === Load Configuration ==
    let config = AureonConfig::load();
    
    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Print configuration summary
    config.print_summary();

    // === Initialize Consensus Engine ===
    let consensus_type = config.get_consensus_type();
    let engine = get_engine(consensus_type);

    // === Initialize Networking ===
    let network = Network::new("aureon-node".to_string(), "1.0.0".to_string());
    let network_clone = network.clone();

    // Add peer addresses from config
    for peer in &config.network.bootstrap_peers {
        network.add_peer(peer, None);
    }

    let listen_addr = format!("{}:{}", config.network.listen_addr, config.network.listen_port);
    thread::spawn(move || {
        network_clone.listen(&listen_addr);
    });

    // === Initialize Block Synchronization State ===
    let sync_state = std::sync::Arc::new(std::sync::Mutex::new(sync::BlockSyncState::new()));
    
    // === Sample Transactions ===
    let transactions = vec![
        Transaction::transfer("Alice".into(), "Bob".into(), 50),
        Transaction::transfer("Charlie".into(), "Dave".into(), 75),
    ];

    // === Set up Database and Trie ===
    let db = Db::open(&config.database.path);
    let mut trie = MerklePatriciaTrie::new();

    // === Initialize Account Balances from Config ===
    for (account, balance) in &config.state.accounts {
        db.put(account.as_bytes(), &balance.to_le_bytes());
        trie.insert(account.as_bytes().to_vec(), balance.to_le_bytes().to_vec());
    }

    println!("Initialized {} genesis accounts", config.state.accounts.len());

    // === Create Blockchain Indexer ===
    let indexer = Arc::new(BlockchainIndexer::new());

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

    // === Index the Block ===
    if let Err(e) = indexer.index_block(block.clone(), 0, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()) {
        eprintln!("Warning: Failed to index block: {}", e);
    }

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
                match WasmRuntime::new(&wasm_bytes) {
                    Ok(wasm_runtime) => {
                        match wasm_runtime.execute_contract(&transactions, 10_000) {
                            Ok(result) => println!("Result: {}\n", result),
                            Err(e) => println!("Execution error: {}\n", e),
                        }
                    }
                    Err(e) => println!("Load error: {}\n", e),
                }
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

    // === Create Transaction Mempool ===
    let mempool = Arc::new(TransactionMempool::new());

    // === Create Arc for database early ===
    let db_arc = Arc::new(db);

    // === Initialize Logging ===
    let _ = logging::init_logging(&config.logging.level);

    // === Initialize Metrics ===
    let metrics = Arc::new(Metrics::new()?);
    
    // Update initial metrics
    if let Ok(Some(height)) = indexer.get_latest_block_number() {
        metrics.chain_height.set(height as i64);
    }
    metrics.pow_difficulty.set(config.consensus.pow_difficulty as i64);
    metrics.pos_validators.set(config.consensus.pos_validator_count as i64);

    // === Start Block Producer ===
    let producer = block_producer::BlockProducer::new(
        mempool.clone(),
        db_arc.clone(),
        indexer.clone(),
        metrics.clone(),
        5000, // Produce a block every 5 seconds
    );
    producer.start();

    // === Start Metrics Tracker ===
    metrics_tracker::MetricsTracker::start_mempool_tracker(
        metrics.clone(),
        mempool.clone(),
        1000, // Update every 1 second
    );

    // === Start REST API Server ===
    let contract_registry = Arc::new(Mutex::new(ContractRegistry::new()));
    
    println!("\n--- Starting REST API Server ---");
    println!("Node is running. Press Ctrl+C to stop.");
    println!("Metrics endpoint: http://{}:8080/metrics", config.api.host);
    println!("Health check: http://{}:8080/health", config.api.host);
    
    // Block on the async API server (will run forever until interrupted)
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        if let Err(e) = start_api_server(db_arc, contract_registry, indexer, mempool, metrics).await {
            eprintln!("API Server error: {}", e);
        }
    });

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
