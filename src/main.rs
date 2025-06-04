mod genesis;
mod crypto;
mod token;
mod staking;
mod state;

use clap::{Parser, Subcommand};
use crypto::derive_address_from_seed;
use genesis::GenesisBlock;
use token::mint_initial_supply;
use staking::apply_reward;
use state::State;

use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Aureon CLI
#[derive(Parser)]
#[command(name = "aureon")]
#[command(about = "CLI tool for Aureon blockchain", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a validator address from a seed
    GenKey {
        #[arg(short, long)]
        seed: String,
    },
    /// Create the genesis block and simulate reward logic
    InitGenesis {
        #[arg(short, long)]
        chain_id: String,
        #[arg(short, long)]
        validators: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenKey { seed } => {
            let addr = derive_address_from_seed(seed);
            println!("🔐 Derived Validator Address: 0x{}", addr);
        }

        Commands::InitGenesis { chain_id, validators } => {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            // Write Genesis Block
            let balances = validators.iter().map(|v| (v.clone(), 1_000_000u64)).collect();
            let genesis = GenesisBlock {
                chain_id: chain_id.clone(),
                timestamp,
                initial_validators: validators.clone(),
                initial_balances: balances,
                nonce: 0,
            };
            let json = serde_json::to_string_pretty(&genesis).unwrap();
            let mut file = File::create("genesis.json").unwrap();
            file.write_all(json.as_bytes()).unwrap();
            println!("✅ Genesis block written to genesis.json");

            // Initialize in-memory state
            let mut state = State::new();

            // Mint initial supply
            mint_initial_supply(&mut state, validators, 1_000_000);
            println!("💰 Initial balances: {:#?}", state.balances);

            // Simulate block rewards at different heights
            for block in [1, 500_000, 1_000_000, 2_000_000] {
                apply_reward(&mut state, &validators[0], block);
                println!("🏆 Reward applied at block {} for {}", block, validators[0]);
            }

            println!("🟢 Final balances: {:#?}", state.balances);
            println!("💸 Total Supply: {}", state.total_supply);
        }
    }
}