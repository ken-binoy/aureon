use crate::consensus::ConsensusType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main configuration structure for Aureon blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AureonConfig {
    pub consensus: ConsensusConfig,
    pub network: NetworkConfig,
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub state: StateConfig,
    pub validator: ValidatorConfig,
    pub logging: LoggingConfig,
}

/// Consensus engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus type: "pow", "pos", or "poa"
    pub engine: String,
    /// PoW difficulty (1-256, higher = harder)
    pub pow_difficulty: u8,
    /// Minimum stake for PoS validators (tokens)
    pub pos_min_stake: u64,
    /// Number of PoS validators
    pub pos_validator_count: usize,
    /// PoA authorized validators
    pub poa_validators: Vec<String>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address for P2P
    pub listen_addr: String,
    /// Listen port for P2P
    pub listen_port: u16,
    /// Bootstrap peers to connect to
    pub bootstrap_peers: Vec<String>,
}

/// REST API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Enable REST API
    pub enabled: bool,
    /// API host (0.0.0.0 = all interfaces)
    pub host: String,
    /// API port
    pub port: u16,
    /// Enable WebSocket support
    pub websocket_enabled: bool,
    /// WebSocket port
    pub websocket_port: u16,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// RocksDB directory path
    pub path: String,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Enable compression
    pub compression: bool,
}

/// Genesis state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Initial account balances: account_name -> balance
    pub accounts: HashMap<String, u64>,
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Validator stake amount
    pub stake: u64,
    /// Validator public key
    pub public_key: String,
    /// Validator operator address
    pub operator_address: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: "debug", "info", "warn", "error"
    pub level: String,
    /// Enable consensus debug logs
    pub consensus_debug: bool,
    /// Enable network trace logs
    pub network_trace: bool,
}

impl Default for AureonConfig {
    fn default() -> Self {
        AureonConfig {
            consensus: ConsensusConfig {
                engine: "pow".to_string(),
                pow_difficulty: 4,
                pos_min_stake: 1000,
                pos_validator_count: 21,
                poa_validators: vec!["alice".to_string(), "bob".to_string()],
            },
            network: NetworkConfig {
                listen_addr: "127.0.0.1".to_string(),
                listen_port: 6000,
                bootstrap_peers: vec![
                    "127.0.0.1:6001".to_string(),
                    "127.0.0.1:6002".to_string(),
                ],
            },
            api: ApiConfig {
                enabled: true,
                host: "0.0.0.0".to_string(),
                port: 8080,
                websocket_enabled: false,
                websocket_port: 8081,
            },
            database: DatabaseConfig {
                path: "aureon_db".to_string(),
                cache_size_mb: 512,
                compression: true,
            },
            state: StateConfig {
                accounts: vec![
                    ("alice".to_string(), 100),
                    ("bob".to_string(), 100),
                    ("charlie".to_string(), 100),
                ]
                .into_iter()
                .collect(),
            },
            validator: ValidatorConfig {
                stake: 10000,
                public_key: String::new(),
                operator_address: "validator1".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                consensus_debug: false,
                network_trace: false,
            },
        }
    }
}

impl AureonConfig {
    /// Load configuration from file or environment
    /// Priority: environment variables > config.toml > defaults
    pub fn load() -> Self {
        // Start with defaults
        let mut config = Self::default();

        // Load from config.toml if it exists
        let config_path = Path::new("config.toml");
        if config_path.exists() {
            if let Ok(contents) = fs::read_to_string(config_path) {
                if let Ok(file_config) = toml::from_str::<AureonConfig>(&contents) {
                    config = file_config;
                } else {
                    eprintln!("Warning: Failed to parse config.toml, using defaults");
                }
            }
        }

        // Override with environment variables
        if let Ok(engine) = std::env::var("AUREON_CONSENSUS_ENGINE") {
            config.consensus.engine = engine;
        }
        if let Ok(difficulty) = std::env::var("AUREON_POW_DIFFICULTY") {
            if let Ok(val) = difficulty.parse() {
                config.consensus.pow_difficulty = val;
            }
        }
        if let Ok(addr) = std::env::var("AUREON_API_HOST") {
            config.api.host = addr;
        }
        if let Ok(port) = std::env::var("AUREON_API_PORT") {
            if let Ok(val) = port.parse() {
                config.api.port = val;
            }
        }
        if let Ok(db_path) = std::env::var("AUREON_DB_PATH") {
            config.database.path = db_path;
        }
        if let Ok(level) = std::env::var("AUREON_LOG_LEVEL") {
            config.logging.level = level;
        }

        config
    }

    /// Get consensus type from engine string
    pub fn get_consensus_type(&self) -> ConsensusType {
        match self.consensus.engine.to_lowercase().as_str() {
            "pos" => ConsensusType::PoS,
            "poa" => ConsensusType::PoA,
            _ => ConsensusType::PoW, // Default to PoW
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate consensus engine
        let valid_engines = vec!["pow", "pos", "poa"];
        if !valid_engines.contains(&self.consensus.engine.to_lowercase().as_str()) {
            return Err(format!(
                "Invalid consensus engine: {}. Must be one of: {:?}",
                self.consensus.engine, valid_engines
            ));
        }

        // Validate PoW difficulty
        if self.consensus.pow_difficulty == 0 {
            return Err("PoW difficulty must be between 1 and 255".to_string());
        }

        // Validate PoS settings
        if self.consensus.pos_validator_count == 0 {
            return Err("PoS validator count must be greater than 0".to_string());
        }

        // Validate PoA validators
        if self.consensus.engine.to_lowercase() == "poa" && self.consensus.poa_validators.is_empty()
        {
            return Err("PoA requires at least one validator".to_string());
        }

        // Validate API port
        if self.api.port == 0 {
            return Err("API port must be greater than 0".to_string());
        }

        // Validate log level
        let valid_levels = vec!["debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            return Err(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.logging.level, valid_levels
            ));
        }

        Ok(())
    }

    /// Print configuration summary
    pub fn print_summary(&self) {
        println!("\n=== Aureon Configuration ===");
        println!("Consensus: {:?}", self.get_consensus_type());
        println!("  Engine: {}", self.consensus.engine);
        if self.consensus.engine.to_lowercase() == "pow" {
            println!("  PoW Difficulty: {}", self.consensus.pow_difficulty);
        }
        if self.consensus.engine.to_lowercase() == "pos" {
            println!("  Min Stake: {} tokens", self.consensus.pos_min_stake);
            println!("  Validator Count: {}", self.consensus.pos_validator_count);
        }
        if self.consensus.engine.to_lowercase() == "poa" {
            println!("  Authorized Validators: {:?}", self.consensus.poa_validators);
        }
        println!("Network:");
        println!("  Listen: {}:{}", self.network.listen_addr, self.network.listen_port);
        println!("  Bootstrap Peers: {}", self.network.bootstrap_peers.len());
        println!("API:");
        println!(
            "  Enabled: {} ({}:{})",
            self.api.enabled, self.api.host, self.api.port
        );
        println!("Database:");
        println!("  Path: {}", self.database.path);
        println!("  Cache: {}MB", self.database.cache_size_mb);
        println!("  Compression: {}", self.database.compression);
        println!("State:");
        println!("  Genesis Accounts: {}", self.state.accounts.len());
        println!("Logging:");
        println!("  Level: {}", self.logging.level);
        println!("=============================\n");
    }
}

/// Backward-compatible function: load consensus type from config
pub fn load_consensus_type() -> ConsensusType {
    let config = AureonConfig::load();
    config.get_consensus_type()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AureonConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_engine() {
        let mut config = AureonConfig::default();
        config.consensus.engine = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_difficulty() {
        let mut config = AureonConfig::default();
        config.consensus.pow_difficulty = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_poa_requires_validators() {
        let mut config = AureonConfig::default();
        config.consensus.engine = "poa".to_string();
        config.consensus.poa_validators.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_get_consensus_type() {
        let mut config = AureonConfig::default();

        config.consensus.engine = "pow".to_string();
        assert!(matches!(config.get_consensus_type(), ConsensusType::PoW));

        config.consensus.engine = "pos".to_string();
        assert!(matches!(config.get_consensus_type(), ConsensusType::PoS));

        config.consensus.engine = "poa".to_string();
        assert!(matches!(config.get_consensus_type(), ConsensusType::PoA));
    }
}