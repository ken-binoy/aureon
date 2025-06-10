use wasmtime::{Caller, Linker};
use super::gas_meter::GasMeter;

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

        // More host functions can be added here similarly,
        // e.g. get_balance, transfer, etc., with gas metering.

        Ok(())
    }
}