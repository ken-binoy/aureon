// api.rs
use axum::{
    extract::{Path, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use axum::serve;

use crate::state::GLOBAL_STATE;
use crate::types::Transaction;
use crate::mempool::MEMPOOL;
use crate::block::produce_block; // now from new module

#[derive(Deserialize)]
pub struct TxRequest {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Serialize)]
pub struct TxResponse {
    status: String,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    balance: u64,
}

async fn get_balance(Path(address): Path<String>) -> Json<BalanceResponse> {
    let state = GLOBAL_STATE.lock().unwrap();
    let balance = *state.balances.get(&address).unwrap_or(&0);
    Json(BalanceResponse { balance })
}

async fn submit_tx(Json(payload): Json<TxRequest>) -> Json<TxResponse> {
    let tx = Transaction {
        from: payload.from,
        to: payload.to,
        amount: payload.amount,
    };
    MEMPOOL.lock().unwrap().push(tx);
    Json(TxResponse {
        status: "Transaction added".to_string(),
    })
}

async fn produce_block_handler() -> Json<TxResponse> {
    // Placeholder: you must gather transactions and state root before using this
    let dummy_txs = vec![Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 10,
    }];
    let pre_root = vec![0u8; 32];
    let post_root = vec![1u8; 32];

    let block = produce_block(dummy_txs, pre_root, post_root);
    println!("Produced Block: {:?}", block);

    Json(TxResponse {
        status: "Block produced successfully".to_string(),
    })
}

pub async fn start_api_server() {
    let app = Router::new()
        .route("/balance/:address", get(get_balance))
        .route("/submit_tx", post(submit_tx))
        .route("/produce-block", post(produce_block_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("📡 Rust API server listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}