#[derive(Debug)]
pub struct GasMeter {
    limit: u64,
    used: u64,
}

impl GasMeter {
    pub fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// Consume gas, return error if limit exceeded
    pub fn consume(&mut self, amount: u64) -> Result<(), &'static str> {
        if self.used + amount > self.limit {
            Err("Out of Gas")
        } else {
            self.used += amount;
            Ok(())
        }
    }

    pub fn gas_used(&self) -> u64 {
        self.used
    }

    pub fn gas_remaining(&self) -> u64 {
        self.limit - self.used
    }
}