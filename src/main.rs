mod genesis;
mod crypto;

use clap::{Parser, Subcommand};
use crypto::derive_address_from_seed;
use genesis::{GenesisBlock};
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
    /// Create the genesis block
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
            let balances = validators
                .iter()
                .map(|v| (v.clone(), 1_000_000u64))
                .collect();

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
        }
    }
}