use std::collections::HashMap;

#[derive(Debug)]
pub struct State {
    pub balances: HashMap<String, u64>,     // Wallet balances
    pub staked: HashMap<String, u64>,       // Staked tokens
    pub total_supply: u64,                  // Current circulating supply
}

impl State {
    pub fn new() -> Self {
        State {
            balances: HashMap::new(),
            staked: HashMap::new(),
            total_supply: 0,
        }
    }

    pub fn mint(&mut self, address: &str, amount: u64) {
        *self.balances.entry(address.to_string()).or_insert(0) += amount;
        self.total_supply += amount;
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), &'static str> {
        let sender_balance = self.balances.entry(from.to_string()).or_insert(0);
        if *sender_balance < amount {
            return Err("Insufficient balance");
        }
        *sender_balance -= amount;
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        Ok(())
    }

    pub fn stake(&mut self, address: &str, amount: u64) -> Result<(), &'static str> {
        let balance = self.balances.entry(address.to_string()).or_insert(0);
        if *balance < amount {
            return Err("Not enough tokens to stake");
        }
        *balance -= amount;
        *self.staked.entry(address.to_string()).or_insert(0) += amount;
        Ok(())
    }
}