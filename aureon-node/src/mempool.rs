use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::types::Transaction;

pub static MEMPOOL: Lazy<Mutex<Vec<Transaction>>> = Lazy::new(|| Mutex::new(Vec::new()));