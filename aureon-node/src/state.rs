use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Default)]
pub struct SharedState {
    pub balances: HashMap<String, u64>,
}

pub static GLOBAL_STATE: Lazy<Mutex<SharedState>> = Lazy::new(|| {
    let mut balances = HashMap::new();
    balances.insert("Alice".to_string(), 100);
    balances.insert("Bob".to_string(), 400);
    balances.insert("Charlie".to_string(), 50);
    balances.insert("Dave".to_string(), 700);
    Mutex::new(SharedState { balances })
});