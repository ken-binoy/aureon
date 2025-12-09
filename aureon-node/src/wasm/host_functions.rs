use wasmtime::{Caller, Linker};
use super::gas_meter::GasMeter;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

/// Context passed to WASM runtime for host function access
#[derive(Clone)]
pub struct WasmContext {
    pub balances: Arc<Mutex<HashMap<String, u64>>>,
    pub storage: Arc<Mutex<HashMap<String, Vec<u8>>>>, // contract storage key-value
}

impl WasmContext {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(Mutex::new(HashMap::new())),
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_balance(&self, address: &str, balance: u64) {
        self.balances.lock().unwrap().insert(address.to_string(), balance);
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        *self.balances.lock().unwrap().get(address).unwrap_or(&0)
    }
}

pub struct HostFunctions;

impl HostFunctions {
    pub fn register(linker: &mut Linker<GasMeter>) -> anyhow::Result<()> {
        // Log host function: charges 10 gas units
        linker.func_wrap("env", "log", |mut caller: Caller<'_, GasMeter>, ptr: i32, len: i32| {
            caller.data_mut().consume(10).map_err(|e| anyhow::anyhow!(e))?;
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| anyhow::anyhow!("failed to find memory"))?;
            let mut buffer = vec![0u8; len as usize];
            memory.read(&caller, ptr as usize, &mut buffer)?;
            let message = String::from_utf8_lossy(&buffer);
            println!("[WASM LOG]: {}", message);
            Ok(())
        })?;

        Ok(())
    }

    /// Register enhanced host functions with context support
    pub fn register_with_context(
        linker: &mut Linker<(GasMeter, WasmContext)>,
    ) -> anyhow::Result<()> {
        // Log host function: charges 10 gas units
        linker.func_wrap(
            "env",
            "log",
            |mut caller: Caller<'_, (GasMeter, WasmContext)>, ptr: i32, len: i32| {
                {
                    let data = caller.data_mut();
                    data.0.consume(10).map_err(|e| anyhow::anyhow!(e))?;
                }

                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("failed to find memory"))?;
                let mut buffer = vec![0u8; len as usize];
                memory.read(&caller, ptr as usize, &mut buffer)?;
                let message = String::from_utf8_lossy(&buffer);
                println!("[WASM LOG]: {}", message);
                Ok(())
            },
        )?;

        // get_balance(address_ptr: i32, address_len: i32) -> u64
        // Charges 20 gas
        linker.func_wrap(
            "env",
            "get_balance",
            |mut caller: Caller<'_, (GasMeter, WasmContext)>,
             addr_ptr: i32,
             addr_len: i32| {
                let context = {
                    let data = caller.data_mut();
                    data.0.consume(20).map_err(|e| anyhow::anyhow!(e))?;
                    data.1.clone()
                };

                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("failed to find memory"))?;

                let mut addr_buffer = vec![0u8; addr_len as usize];
                memory.read(&caller, addr_ptr as usize, &mut addr_buffer)?;
                let address = String::from_utf8(addr_buffer)?;

                let balance = context.get_balance(&address);
                Ok(balance)
            },
        )?;

        // transfer(from_ptr: i32, from_len: i32, to_ptr: i32, to_len: i32, amount: u64) -> i32
        // Returns 0 on success, 1 on failure
        // Charges 50 gas
        linker.func_wrap(
            "env",
            "transfer",
            |mut caller: Caller<'_, (GasMeter, WasmContext)>,
             from_ptr: i32,
             from_len: i32,
             to_ptr: i32,
             to_len: i32,
             amount: u64| {
                let context = {
                    let data = caller.data_mut();
                    data.0.consume(50).map_err(|e| anyhow::anyhow!(e))?;
                    data.1.clone()
                };

                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("failed to find memory"))?;

                // Read from address
                let mut from_buffer = vec![0u8; from_len as usize];
                memory.read(&caller, from_ptr as usize, &mut from_buffer)?;
                let from = String::from_utf8(from_buffer)?;

                // Read to address
                let mut to_buffer = vec![0u8; to_len as usize];
                memory.read(&caller, to_ptr as usize, &mut to_buffer)?;
                let to = String::from_utf8(to_buffer)?;

                // Check balance
                let from_balance = context.get_balance(&from);
                if from_balance < amount {
                    return Ok(1i32); // Insufficient balance
                }

                // Update balances
                context.set_balance(&from, from_balance - amount);
                let to_balance = context.get_balance(&to);
                context.set_balance(&to, to_balance + amount);

                Ok(0i32) // Success
            },
        )?;

        // storage_read(key_ptr: i32, key_len: i32, value_ptr: i32, value_max_len: i32) -> i32
        // Returns actual length of value read, or -1 if not found
        // Charges 15 gas
        linker.func_wrap(
            "env",
            "storage_read",
            |mut caller: Caller<'_, (GasMeter, WasmContext)>,
             key_ptr: i32,
             key_len: i32,
             value_ptr: i32,
             value_max_len: i32| {
                let context = {
                    let data = caller.data_mut();
                    data.0.consume(15).map_err(|e| anyhow::anyhow!(e))?;
                    data.1.clone()
                };

                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("failed to find memory"))?;

                // Read key
                let mut key_buffer = vec![0u8; key_len as usize];
                memory.read(&caller, key_ptr as usize, &mut key_buffer)?;
                let key = String::from_utf8(key_buffer)?;

                // Get value from storage
                let storage = context.storage.lock().unwrap();
                if let Some(value) = storage.get(&key) {
                    let value_len = std::cmp::min(value.len(), value_max_len as usize);
                    memory.write(
                        &mut caller,
                        value_ptr as usize,
                        &value[0..value_len],
                    )?;
                    Ok(value_len as i32)
                } else {
                    Ok(-1i32) // Key not found
                }
            },
        )?;

        // storage_write(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32) -> i32
        // Returns 0 on success, 1 on failure
        // Charges 30 gas
        linker.func_wrap(
            "env",
            "storage_write",
            |mut caller: Caller<'_, (GasMeter, WasmContext)>,
             key_ptr: i32,
             key_len: i32,
             value_ptr: i32,
             value_len: i32| {
                let context = {
                    let data = caller.data_mut();
                    data.0.consume(30).map_err(|e| anyhow::anyhow!(e))?;
                    data.1.clone()
                };

                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("failed to find memory"))?;

                // Read key
                let mut key_buffer = vec![0u8; key_len as usize];
                memory.read(&caller, key_ptr as usize, &mut key_buffer)?;
                let key = String::from_utf8(key_buffer)?;

                // Read value
                let mut value_buffer = vec![0u8; value_len as usize];
                memory.read(&caller, value_ptr as usize, &mut value_buffer)?;

                // Store in storage
                context.storage.lock().unwrap().insert(key, value_buffer);
                Ok(0i32)
            },
        )?;

        Ok(())
    }
}
