use axum::{
    extract::{Path, Json, State as AxumState},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use axum::serve;
use hex;

use crate::types::Transaction;
use crate::db::Db;
use crate::contract_registry::ContractRegistry;
use crate::wasm::WasmRuntime;
use crate::indexer::BlockchainIndexer;
use crate::mempool::TransactionMempool;
use crate::metrics::Metrics;
use crate::monitoring::monitoring_router;

// ============================================================================
// Request/Response Structs
// ============================================================================

#[derive(Deserialize, Clone)]
pub struct BalanceRequest {
    pub address: String,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
}

#[derive(Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Deserialize)]
pub struct SignedTransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub public_key: String,  // Hex-encoded Ed25519 public key
    pub signature: String,   // Hex-encoded Ed25519 signature
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct BlockResponse {
    pub hash: String,
    pub block_number: u64,
    pub transaction_count: usize,
}

#[derive(Deserialize)]
pub struct ContractDeployRequest {
    pub code: Vec<u8>,
    pub gas_limit: u64,
}

#[derive(Serialize)]
pub struct ContractDeployResponse {
    pub address: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct ContractCallRequest {
    pub contract_address: String,
    pub function: String,
    pub args: String,
    pub gas_limit: u64,
}

#[derive(Serialize)]
pub struct ContractCallResponse {
    pub success: bool,
    pub output: String,
    pub gas_used: u64,
}

#[derive(Serialize)]
pub struct ChainInfoResponse {
    pub chain_name: String,
    pub best_block_number: u64,
    pub best_block_hash: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Clone)]
pub struct BlockEvent {
    pub event_type: String,
    pub block_hash: String,
    pub block_number: u64,
    pub timestamp: u64,
}

#[derive(Serialize, Clone)]
pub struct TransactionEvent {
    pub event_type: String,
    pub tx_hash: String,
    pub from: String,
    pub block_number: u64,
}

// ============================================================================
// Shared State (passed to handlers via Axum State)
// ============================================================================

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Db>,
    pub contract_registry: Arc<Mutex<ContractRegistry>>,
    pub indexer: Arc<BlockchainIndexer>,
    pub mempool: Arc<TransactionMempool>,
    pub metrics: Arc<Metrics>,
}

// ============================================================================
// Handler Functions
// ============================================================================

async fn get_balance(
    Path(address): Path<String>,
    AxumState(state): AxumState<ApiState>,
) -> Json<BalanceResponse> {
    let balance = state.db.get(address.as_bytes())
        .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap_or_default()))
        .unwrap_or(0);

    Json(BalanceResponse {
        address: address.clone(),
        balance,
    })
}

async fn submit_transaction(
    AxumState(state): AxumState<ApiState>,
    Json(payload): Json<TransactionRequest>,
) -> Json<TransactionResponse> {
    // Validate transaction
    if payload.from.is_empty() || payload.to.is_empty() {
        return Json(TransactionResponse {
            status: "error".to_string(),
            message: "Invalid sender or recipient".to_string(),
        });
    }

    if payload.amount == 0 {
        return Json(TransactionResponse {
            status: "error".to_string(),
            message: "Amount must be greater than 0".to_string(),
        });
    }

    // Create Transaction and add to mempool
    let tx = Transaction::transfer(payload.from.clone(), payload.to.clone(), payload.amount);

    match state.mempool.add_transaction(tx) {
        Ok(tx_hash) => {
            Json(TransactionResponse {
                status: "success".to_string(),
                message: format!("Transaction {} added to mempool", tx_hash),
            })
        }
        Err(e) => {
            Json(TransactionResponse {
                status: "error".to_string(),
                message: format!("Failed to add transaction to mempool: {}", e),
            })
        }
    }
}

async fn submit_signed_transaction(
    AxumState(state): AxumState<ApiState>,
    Json(payload): Json<SignedTransactionRequest>,
) -> Json<TransactionResponse> {
    // Validate transaction
    if payload.from.is_empty() || payload.to.is_empty() {
        return Json(TransactionResponse {
            status: "error".to_string(),
            message: "Invalid sender or recipient".to_string(),
        });
    }

    if payload.amount == 0 {
        return Json(TransactionResponse {
            status: "error".to_string(),
            message: "Amount must be greater than 0".to_string(),
        });
    }

    // Decode public key and signature from hex
    let public_key = match hex::decode(&payload.public_key) {
        Ok(pk) => pk,
        Err(_) => {
            return Json(TransactionResponse {
                status: "error".to_string(),
                message: "Invalid public key format (must be hex)".to_string(),
            })
        }
    };

    let signature = match hex::decode(&payload.signature) {
        Ok(sig) => sig,
        Err(_) => {
            return Json(TransactionResponse {
                status: "error".to_string(),
                message: "Invalid signature format (must be hex)".to_string(),
            })
        }
    };

    // Create signed transaction
    let mut tx = Transaction::transfer(payload.from.clone(), payload.to.clone(), payload.amount);
    tx.nonce = payload.nonce;
    tx.public_key = public_key;
    tx.signature = signature;

    // Add to mempool (signature verification happens here)
    match state.mempool.add_transaction(tx) {
        Ok(tx_hash) => {
            Json(TransactionResponse {
                status: "success".to_string(),
                message: format!("Signed transaction {} added to mempool", tx_hash),
            })
        }
        Err(e) => {
            Json(TransactionResponse {
                status: "error".to_string(),
                message: format!("Failed to add transaction: {}", e),
            })
        }
    }
}

async fn get_block(
    Path(block_hash): Path<String>,
    AxumState(state): AxumState<ApiState>,
) -> Json<serde_json::Value> {
    match state.indexer.get_block(&block_hash) {
        Ok(Some(block_entry)) => {
            let tx_count = block_entry.block.transactions.len();
            Json(serde_json::json!({
                "hash": block_entry.block.hash,
                "number": block_entry.block_number,
                "timestamp": block_entry.timestamp,
                "transactions": tx_count,
                "previous_hash": block_entry.block.previous_hash,
                "nonce": block_entry.block.nonce
            }))
        }
        Ok(None) => {
            Json(serde_json::json!({
                "error": "Block not found"
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "error": format!("Failed to query block: {}", e)
            }))
        }
    }
}

async fn get_transaction(
    Path(tx_hash): Path<String>,
    AxumState(state): AxumState<ApiState>,
) -> Json<serde_json::Value> {
    match state.indexer.get_transaction(&tx_hash) {
        Ok(Some(tx_entry)) => {
            let tx = &tx_entry.transaction;
            Json(serde_json::json!({
                "hash": tx_hash,
                "from": tx.from,
                "block_hash": tx_entry.block_hash,
                "block_number": tx_entry.block_number,
                "tx_index": tx_entry.tx_index,
                "gas_price": tx.gas_price,
                "nonce": tx.nonce
            }))
        }
        Ok(None) => {
            Json(serde_json::json!({
                "error": "Transaction not found"
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "error": format!("Failed to query transaction: {}", e)
            }))
        }
    }
}

async fn get_chain_head(
    AxumState(state): AxumState<ApiState>,
) -> Json<ChainInfoResponse> {
    let best_block_number = state.indexer.get_latest_block_number()
        .unwrap_or(None)
        .unwrap_or(0);
    let best_block_hash = state.indexer.get_latest_block_hash()
        .unwrap_or(None)
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000000000000000000000000000".to_string());

    Json(ChainInfoResponse {
        chain_name: "Aureon".to_string(),
        best_block_number,
        best_block_hash,
    })
}

async fn deploy_contract(
    AxumState(state): AxumState<ApiState>,
    Json(payload): Json<ContractDeployRequest>,
) -> Json<ContractDeployResponse> {
    // Validate code is not empty
    if payload.code.is_empty() {
        return Json(ContractDeployResponse {
            address: String::new(),
            status: "failed: empty code".to_string(),
        });
    }

    // Try to validate WASM code
    match WasmRuntime::new(&payload.code) {
        Ok(_) => {
            // Deploy contract and store in registry
            let mut registry = state.contract_registry.lock().unwrap();
            let address = registry.deploy(payload.code.clone());

            Json(ContractDeployResponse {
                address,
                status: "deployed".to_string(),
            })
        }
        Err(e) => {
            Json(ContractDeployResponse {
                address: String::new(),
                status: format!("failed: {}", e),
            })
        }
    }
}

async fn call_contract(
    AxumState(state): AxumState<ApiState>,
    Json(payload): Json<ContractCallRequest>,
) -> Json<ContractCallResponse> {
    // Verify contract exists
    let registry = state.contract_registry.lock().unwrap();
    let code = match registry.get_contract(&payload.contract_address) {
        Some(code) => code,
        None => {
            return Json(ContractCallResponse {
                success: false,
                output: "Contract not found".to_string(),
                gas_used: 0,
            });
        }
    };
    drop(registry); // Release lock before executing

    // Execute contract
    match WasmRuntime::new(&code) {
        Ok(runtime) => {
            match runtime.execute_contract_with_context(payload.gas_limit, Default::default()) {
                Ok(result) => {
                    Json(ContractCallResponse {
                        success: result.success,
                        output: result.output,
                        gas_used: result.gas_used,
                    })
                }
                Err(e) => {
                    Json(ContractCallResponse {
                        success: false,
                        output: format!("Execution error: {}", e),
                        gas_used: 0,
                    })
                }
            }
        }
        Err(e) => {
            Json(ContractCallResponse {
                success: false,
                output: format!("Failed to load contract: {}", e),
                gas_used: 0,
            })
        }
    }
}

// ============================================================================
// WebSocket Handler (Phase 5.2)
// ============================================================================

async fn subscribe(
    AxumState(state): AxumState<ApiState>,
) -> Json<serde_json::Value> {
    // Phase 5.2: Placeholder for WebSocket subscription
    // In production, this would upgrade to WebSocket and stream events
    // For now, return available subscription topics
    
    let block_count = state.indexer.get_block_count().unwrap_or(0);
    let tx_count = state.indexer.get_transaction_count().unwrap_or(0);
    
    Json(serde_json::json!({
        "status": "WebSocket subscriptions enabled (Phase 5.2)",
        "available_topics": [
            "blocks",
            "transactions",
            "contracts"
        ],
        "current_state": {
            "blocks": block_count,
            "transactions": tx_count
        },
        "info": "Connect to ws:// endpoint for real-time events (Phase 5.3)"
    }))
}

async fn get_mempool(
    AxumState(state): AxumState<ApiState>,
) -> Json<serde_json::Value> {
    // Return mempool statistics and pending transactions
    match state.mempool.stats() {
        Ok(stats) => {
            Json(serde_json::json!({
                "status": "ok",
                "pending_transactions": stats.transaction_count,
                "total_gas": stats.total_pending_gas,
                "utilization_percent": stats.utilization_percent,
                "max_capacity": stats.max_capacity,
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to get mempool stats: {}", e)
            }))
        }
    }
}

// ============================================================================
// API Server Setup
// ============================================================================

pub async fn start_api_server(
    db: Arc<Db>,
    contract_registry: Arc<Mutex<ContractRegistry>>,
    indexer: Arc<BlockchainIndexer>,
    mempool: Arc<TransactionMempool>,
    metrics: Arc<Metrics>,
) -> anyhow::Result<()> {
    let state = ApiState {
        db,
        contract_registry,
        indexer,
        mempool,
        metrics: metrics.clone(),
    };

    let app = Router::new()
        // Balance queries
        .route("/balance/:address", get(get_balance))
        // Transaction submission
        .route("/submit-tx", post(submit_transaction))
        .route("/submit-signed-tx", post(submit_signed_transaction))
        // Block queries
        .route("/block/:hash", get(get_block))
        .route("/tx/:hash", get(get_transaction))
        .route("/chain/head", get(get_chain_head))
        // Contract operations
        .route("/contract/deploy", post(deploy_contract))
        .route("/contract/call", post(call_contract))
        // Event subscriptions (Phase 5.2)
        .route("/subscribe", get(subscribe))
        // Mempool (Phase 5.3)
        .route("/mempool", get(get_mempool))
        .with_state(state)
        .nest("/", monitoring_router(metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("📡 Aureon API listening on http://0.0.0.0:8080 (access via http://127.0.0.1:8080 locally)");
    println!("📊 Prometheus metrics: http://0.0.0.0:8080/metrics");
    println!("💚 Health check: http://0.0.0.0:8080/health");

    let listener = TcpListener::bind(&addr).await?;
    serve(listener, app).await?;

    Ok(())
}
