use wasmtime::{Engine, Store, Module, Linker};
use super::gas_meter::GasMeter;
use super::host_functions::HostFunctions;
use crate::types::Transaction;

pub struct WasmRuntime {
    engine: Engine,
    module: Module,
}

impl WasmRuntime {
    pub fn new(wasm_bytes: &[u8]) -> anyhow::Result<Self> {
        let engine = Engine::default();
        let module = Module::from_binary(&engine, wasm_bytes)?;
        Ok(Self { engine, module })
    }

    pub fn execute_contract(
        &self,
        _input_txs: &[Transaction],
        gas_limit: u64,
    ) -> anyhow::Result<String> {
        let mut store = Store::new(&self.engine, GasMeter::new(gas_limit));
        let mut linker = Linker::new(&self.engine);

        // Register host functions with gas metering
        HostFunctions::register(&mut linker)?;

        let instance = linker.instantiate(&mut store, &self.module)?;

        let run_func = instance.get_func(&mut store, "run")
            .ok_or_else(|| anyhow::anyhow!("Function 'run' not found"))?;

        // Call the run function in WASM
        run_func.call(&mut store, &[], &mut [])?;

        println!("Gas used: {}", store.data().gas_used());

        Ok("Contract executed successfully".to_string())
    }
}