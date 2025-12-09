//! SPV Client API Routes
//! 
//! HTTP API endpoints for light client operations
//! Provides endpoints for:
//! - Header synchronization
//! - Transaction verification
//! - Balance queries
//! - Transaction submission

use crate::light_block_header::LightBlockHeader;
use crate::merkle_tree::MerkleInclusionProof;
use crate::spv_client::SpvClient;
use crate::state_compression::{CompressedAccount, StateCompressionManager};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// API request to add a new block header
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddHeaderRequest {
    pub height: u64,
    pub prev_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u64,
}

impl AddHeaderRequest {
    /// Convert to LightBlockHeader
    pub fn to_header(&self) -> LightBlockHeader {
        LightBlockHeader::new(
            self.height,
            self.prev_hash.clone(),
            self.merkle_root.clone(),
            self.timestamp,
            self.difficulty,
            self.nonce,
        )
    }
}

/// API request to add multiple headers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddHeadersRequest {
    pub headers: Vec<AddHeaderRequest>,
}

/// API response for header addition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddHeaderResponse {
    pub success: bool,
    pub message: String,
    pub height: Option<u64>,
}

/// API request to verify a transaction
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifyTransactionRequest {
    pub tx_hash: String,
    pub merkle_root: String,
    pub proof_path: Vec<ProofElement>,
    pub tx_index: usize,
}

/// Single element in a merkle proof path
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofElement {
    pub hash: String,
    pub is_left: bool,
}

/// API response for transaction verification
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifyTransactionResponse {
    pub valid: bool,
    pub message: String,
    pub merkle_root: Option<String>,
}

/// API response for header info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeaderResponse {
    pub height: u64,
    pub block_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub difficulty: u32,
    pub prev_hash: String,
}

impl From<&LightBlockHeader> for HeaderResponse {
    fn from(header: &LightBlockHeader) -> Self {
        HeaderResponse {
            height: header.height,
            block_hash: header.block_hash.clone(),
            merkle_root: header.merkle_root.clone(),
            timestamp: header.timestamp,
            difficulty: header.difficulty,
            prev_hash: header.prev_hash.clone(),
        }
    }
}

/// API response for chain status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainStatusResponse {
    pub synced: bool,
    pub latest_height: u64,
    pub header_count: u64,
    pub total_size_bytes: usize,
}

/// API response for storage efficiency
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageEfficiencyResponse {
    pub spv_storage_bytes: usize,
    pub full_node_equivalent_bytes: usize,
    pub space_savings_percentage: f64,
}

/// API server for SPV client operations
pub struct SpvApiServer {
    spv_client: Arc<Mutex<SpvClient>>,
    state_manager: Arc<Mutex<StateCompressionManager>>,
}

impl SpvApiServer {
    /// Create a new SPV API server
    pub fn new(spv_client: Arc<Mutex<SpvClient>>, state_manager: Arc<Mutex<StateCompressionManager>>) -> Self {
        SpvApiServer {
            spv_client,
            state_manager,
        }
    }

    /// Handle add header request
    pub fn handle_add_header(&self, req: AddHeaderRequest) -> AddHeaderResponse {
        let mut client = self.spv_client.lock().unwrap();
        let header = req.to_header();
        
        let success = client.add_header(header.clone());
        
        if success {
            AddHeaderResponse {
                success: true,
                message: "Header added successfully".to_string(),
                height: Some(header.height),
            }
        } else {
            AddHeaderResponse {
                success: false,
                message: "Failed to add header".to_string(),
                height: None,
            }
        }
    }

    /// Handle add headers batch request
    pub fn handle_add_headers(&self, req: AddHeadersRequest) -> AddHeaderResponse {
        let mut client = self.spv_client.lock().unwrap();
        let headers: Vec<LightBlockHeader> = req.headers.iter().map(|h| h.to_header()).collect();
        
        let count = client.add_headers(headers.clone());
        
        AddHeaderResponse {
            success: count > 0,
            message: format!("Added {} headers successfully", count),
            height: headers.last().map(|h| h.height),
        }
    }

    /// Get latest header
    pub fn handle_get_latest_header(&self) -> Option<HeaderResponse> {
        let client = self.spv_client.lock().unwrap();
        client.get_latest_header().map(|h| h.into())
    }

    /// Get header by hash
    pub fn handle_get_header(&self, block_hash: &str) -> Option<HeaderResponse> {
        let client = self.spv_client.lock().unwrap();
        client.get_header(block_hash).map(|h| h.into())
    }

    /// Get headers in range
    pub fn handle_get_headers_range(&self, start_height: u64, end_height: u64) -> Vec<HeaderResponse> {
        let client = self.spv_client.lock().unwrap();
        client
            .get_headers_in_range(start_height, end_height)
            .into_iter()
            .map(|h| h.into())
            .collect()
    }

    /// Get chain sync status
    pub fn handle_get_chain_status(&self) -> ChainStatusResponse {
        let client = self.spv_client.lock().unwrap();
        
        let latest_height = client.get_latest_header().map(|h| h.height).unwrap_or(0);
        let header_count = client.header_count();
        let total_size = (header_count * 220) as usize;  // ~220 bytes per header
        
        ChainStatusResponse {
            synced: header_count > 0,
            latest_height,
            header_count: header_count as u64,
            total_size_bytes: total_size,
        }
    }

    /// Get storage efficiency metrics
    pub fn handle_get_storage_efficiency(&self) -> StorageEfficiencyResponse {
        let client = self.spv_client.lock().unwrap();
        
        let spv_size = (client.spv_storage_used()) as usize;
        let full_node_size = (client.estimated_full_node_size()) as usize;
        let savings = client.space_savings_percentage();
        
        StorageEfficiencyResponse {
            spv_storage_bytes: spv_size,
            full_node_equivalent_bytes: full_node_size,
            space_savings_percentage: savings,
        }
    }

    /// Handle verify transaction request
    pub fn handle_verify_transaction(&self, req: VerifyTransactionRequest) -> VerifyTransactionResponse {
        let client = self.spv_client.lock().unwrap();
        
        // Convert API proof format to internal format
        let proof = MerkleInclusionProof {
            tx_hash: req.tx_hash.clone(),
            merkle_root: req.merkle_root.clone(),
            proof_path: req.proof_path.into_iter().map(|p| {
                crate::merkle_tree::MerkleProofElement {
                    hash: p.hash,
                    is_left: p.is_left,
                }
            }).collect(),
            tx_index: req.tx_index,
        };
        
        // Use the merkle root as the block hash for verification
        let result = client.verify_transaction(&req.merkle_root, &req.tx_hash, &proof);
        
        let (valid, message) = match result {
            crate::spv_client::VerificationResult::Valid => {
                (true, "Transaction verified and confirmed".to_string())
            }
            crate::spv_client::VerificationResult::Invalid => {
                (false, "Transaction verification failed".to_string())
            }
            crate::spv_client::VerificationResult::InsufficientConfirmations => {
                (false, "Transaction has insufficient confirmations".to_string())
            }
            crate::spv_client::VerificationResult::MalformedProof => {
                (false, "Invalid proof format".to_string())
            }
        };
        
        VerifyTransactionResponse {
            valid,
            message,
            merkle_root: if valid { Some(req.merkle_root) } else { None },
        }
    }

    /// Add compressed account state
    pub fn handle_add_compressed_account(&self, _address: String, account: CompressedAccount) {
        let mut manager = self.state_manager.lock().unwrap();
        
        // Get or create a snapshot at the current height
        let client = self.spv_client.lock().unwrap();
        let height = client.get_latest_header().map(|h| h.height).unwrap_or(0);
        drop(client);
        
        let mut snapshot = crate::state_compression::CompressedStateSnapshot::new(
            height,
            String::new(),
            String::new(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        
        snapshot.add_account(account);
        manager.add_snapshot(snapshot);
    }

    /// Get compressed account state
    pub fn handle_get_compressed_account(&self, height: u64, address: &str) -> Option<CompressedAccount> {
        let manager = self.state_manager.lock().unwrap();
        
        manager.get_snapshot(height)
            .and_then(|snapshot| snapshot.get_account(address).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_header_request_to_header() {
        let req = AddHeaderRequest {
            height: 100,
            prev_hash: "prev_hash".to_string(),
            merkle_root: "merkle_root".to_string(),
            timestamp: 1234567890,
            difficulty: 1000,
            nonce: 42,
        };

        let header = req.to_header();
        assert_eq!(header.height, 100);
        assert_eq!(header.timestamp, 1234567890);
    }

    #[test]
    fn test_header_response_from_light_block_header() {
        let header = LightBlockHeader::new(
            100,
            "prev_hash".to_string(),
            "merkle_root".to_string(),
            1234567890,
            1000,
            42,
        );

        let response: HeaderResponse = (&header).into();
        assert_eq!(response.height, 100);
        assert_eq!(response.timestamp, 1234567890);
    }

    #[test]
    fn test_spv_api_server_creation() {
        let client = Arc::new(Mutex::new(SpvClient::new(6)));
        let state_manager = Arc::new(Mutex::new(StateCompressionManager::new()));
        
        let _server = SpvApiServer::new(client, state_manager);
    }

    #[test]
    fn test_chain_status_response_creation() {
        let response = ChainStatusResponse {
            synced: true,
            latest_height: 100,
            header_count: 100,
            total_size_bytes: 22000,
        };

        assert_eq!(response.latest_height, 100);
        assert_eq!(response.header_count, 100);
    }

    #[test]
    fn test_storage_efficiency_response() {
        let response = StorageEfficiencyResponse {
            spv_storage_bytes: 22000,
            full_node_equivalent_bytes: 1000000,
            space_savings_percentage: 97.8,
        };

        assert!(response.space_savings_percentage > 90.0);
    }

    #[test]
    fn test_verify_transaction_response() {
        let response = VerifyTransactionResponse {
            valid: true,
            message: "Transaction verified".to_string(),
            merkle_root: Some("root_hash".to_string()),
        };

        assert!(response.valid);
    }

    #[test]
    fn test_add_header_response_success() {
        let response = AddHeaderResponse {
            success: true,
            message: "Success".to_string(),
            height: Some(100),
        };

        assert!(response.success);
    }

    #[test]
    fn test_add_header_response_failure() {
        let response = AddHeaderResponse {
            success: false,
            message: "Failed".to_string(),
            height: None,
        };

        assert!(!response.success);
    }

    #[test]
    fn test_add_headers_request() {
        let headers = vec![
            AddHeaderRequest {
                height: 100,
                prev_hash: "prev_hash_1".to_string(),
                merkle_root: "merkle_root_1".to_string(),
                timestamp: 1234567890,
                difficulty: 1000,
                nonce: 42,
            },
            AddHeaderRequest {
                height: 101,
                prev_hash: "prev_hash_2".to_string(),
                merkle_root: "merkle_root_2".to_string(),
                timestamp: 1234567891,
                difficulty: 1000,
                nonce: 43,
            },
        ];

        let req = AddHeadersRequest { headers };
        assert_eq!(req.headers.len(), 2);
    }

    #[test]
    fn test_spv_api_server_handle_add_header() {
        let client = Arc::new(Mutex::new(SpvClient::new(6)));
        let state_manager = Arc::new(Mutex::new(StateCompressionManager::new()));
        let server = SpvApiServer::new(client, state_manager);

        let req = AddHeaderRequest {
            height: 100,
            prev_hash: String::new(),
            merkle_root: "merkle_root".to_string(),
            timestamp: 1234567890,
            difficulty: 1000,
            nonce: 42,
        };

        let response = server.handle_add_header(req);
        assert!(response.success);
        assert_eq!(response.height, Some(100));
    }

    #[test]
    fn test_spv_api_server_get_chain_status() {
        let client = Arc::new(Mutex::new(SpvClient::new(6)));
        let state_manager = Arc::new(Mutex::new(StateCompressionManager::new()));
        let server = SpvApiServer::new(client, state_manager);

        let status = server.handle_get_chain_status();
        assert!(!status.synced);  // No headers added yet
    }

    #[test]
    fn test_spv_api_server_get_storage_efficiency() {
        let client = Arc::new(Mutex::new(SpvClient::new(6)));
        let state_manager = Arc::new(Mutex::new(StateCompressionManager::new()));
        let server = SpvApiServer::new(client, state_manager);

        let efficiency = server.handle_get_storage_efficiency();
        assert_eq!(efficiency.spv_storage_bytes, 0);  // No headers yet
    }

    #[test]
    fn test_spv_api_server_verify_transaction() {
        let client = Arc::new(Mutex::new(SpvClient::new(6)));
        let state_manager = Arc::new(Mutex::new(StateCompressionManager::new()));
        let server = SpvApiServer::new(client, state_manager);

        let req = VerifyTransactionRequest {
            tx_hash: "tx_hash_123".to_string(),
            merkle_root: "root".to_string(),
            proof_path: vec![],
            tx_index: 0,
        };

        let response = server.handle_verify_transaction(req);
        // Should fail because no headers in SPV client
        assert!(!response.valid);
    }
}
