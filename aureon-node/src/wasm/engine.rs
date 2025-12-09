use wasmtime::{Engine, Store, Module, Linker};
use super::gas_meter::GasMeter;
use super::host_functions::{HostFunctions, WasmContext};
use crate::types::Transaction;
use std::collections::HashMap;

pub struct WasmRuntime {
    engine: Engine,
    module: Module,
}

pub struct ContractExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub output: String,
    pub state_changes: HashMap<String, u64>, // Balance changes
    pub storage_changes: HashMap<String, Vec<u8>>, // Storage changes
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

    /// Execute contract with state context support
    pub fn execute_contract_with_context(
        &self,
        gas_limit: u64,
        initial_balances: HashMap<String, u64>,
    ) -> anyhow::Result<ContractExecutionResult> {
        let context = WasmContext::new();
        
        // Initialize balances
        for (address, balance) in initial_balances {
            context.set_balance(&address, balance);
        }

        let mut store = Store::new(&self.engine, (GasMeter::new(gas_limit), context.clone()));
        let mut linker = Linker::new(&self.engine);

        // Register enhanced host functions with context
        HostFunctions::register_with_context(&mut linker)?;

        let instance = linker.instantiate(&mut store, &self.module)?;

        let run_func = instance.get_func(&mut store, "run")
            .ok_or_else(|| anyhow::anyhow!("Function 'run' not found"))?;

        // Call the run function
        run_func.call(&mut store, &[], &mut [])?;

        let (gas_meter, context) = store.into_data();
        let gas_used = gas_meter.gas_used();

        Ok(ContractExecutionResult {
            success: true,
            gas_used,
            output: "Contract executed successfully".to_string(),
            state_changes: context.balances.lock().unwrap().clone(),
            storage_changes: context.storage.lock().unwrap().clone(),
        })
    }
}