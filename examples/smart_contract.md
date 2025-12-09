# Example: Smart Contract Deployment & Execution

Learn how to deploy and execute WebAssembly smart contracts on Aureon.

## Overview

This example demonstrates:
1. Deploying a WASM smart contract
2. Calling contract functions
3. Managing gas costs
4. Handling execution results

## Smart Contract Example

### Simple Sum Contract (WASM)

```wasm
(module
  (func $add (param i32 i32) (result i32)
    (i32.add (local.get 0) (local.get 1))
  )
  
  (func $multiply (param i32 i32) (result i32)
    (i32.mul (local.get 0) (local.get 1))
  )
  
  (export "add" (func $add))
  (export "multiply" (func $multiply))
)
```

### Rust Code to Deploy & Execute

```rust
use aureon_node::wasm::engine::WasmEngine;
use aureon_node::contracts::registry::ContractRegistry;

fn main() -> Result<(), String> {
    // Initialize
    let mut registry = ContractRegistry::new();
    let engine = WasmEngine::new();
    
    // Sample WASM bytecode (compiled from above)
    let contract_code = vec![
        0x00, 0x61, 0x73, 0x6d,  // WASM magic number
        0x01, 0x00, 0x00, 0x00,  // Version
        // ... rest of compiled bytecode
    ];
    
    // Deploy contract
    let contract_addr = registry.deploy(contract_code.clone())?;
    println!("Contract deployed at: {}", contract_addr);
    
    // Execute: add(5, 3)
    let result_add = engine.execute(&contract_code, "add", &[5, 3])?;
    println!("add(5, 3) = {}", result_add);
    assert_eq!(result_add, 8);
    
    // Execute: multiply(4, 6)
    let result_mul = engine.execute(&contract_code, "multiply", &[4, 6])?;
    println!("multiply(4, 6) = {}", result_mul);
    assert_eq!(result_mul, 24);
    
    // Verify contract exists
    assert!(registry.contract_exists(&contract_addr));
    
    println!("\n✓ Smart contracts executed successfully!");
    
    Ok(())
}
```

## Running This Example

```bash
# Compile the WASM contract
rustc --target wasm32-unknown-unknown sum.rs -o sum.wasm

# Run the example
cargo run --example smart_contract

# Expected output:
# Contract deployed at: 0x1a2b3c4d...
# add(5, 3) = 8
# multiply(4, 6) = 24
#
# ✓ Smart contracts executed successfully!
```

## Advanced: Contract with State

```rust
// Contract that stores and retrieves a value
(module
  (memory 1)
  (data (i32.const 0) "state")
  
  (func $store (param $value i32)
    (i32.store (i32.const 0) (local.get $value))
  )
  
  (func $load (result i32)
    (i32.load (i32.const 0))
  )
  
  (export "store" (func $store))
  (export "load" (func $load))
)
```

## Gas Metering

Each operation has a gas cost:

```rust
pub const GAS_PER_OPERATION: u64 = 1_000;
pub const MAX_GAS_PER_CONTRACT: u64 = 100_000_000;

// Example: add() costs ~6,000 gas (6 operations)
// Example: multiply() costs ~8,000 gas (8 operations)
```

## Contract Registry

Store and retrieve contracts:

```rust
// Deploy
let addr = registry.deploy(code)?;

// Retrieve
let code = registry.get_contract(&addr)?;

// Check exists
if registry.contract_exists(&addr) {
    println!("Contract found!");
}
```

## Error Handling

```rust
fn safe_execute(
    engine: &WasmEngine,
    code: &[u8],
    function: &str,
    args: &[i32],
) -> Result<i32, String> {
    // Validate function name
    if function.is_empty() {
        return Err("Function name cannot be empty".to_string());
    }
    
    // Validate code
    if code.is_empty() {
        return Err("Contract code cannot be empty".to_string());
    }
    
    // Execute with error handling
    match engine.execute(code, function, args) {
        Ok(result) => {
            println!("Execution successful: {}", result);
            Ok(result)
        }
        Err(e) => {
            println!("Execution failed: {}", e);
            Err(e)
        }
    }
}
```

## Contract Lifecycle

```
1. Compilation (Rust/WAT -> WASM bytecode)
   ↓
2. Deployment (Register in blockchain)
   ↓
3. Execution (Call functions)
   ↓
4. Gas Accounting (Track costs)
   ↓
5. State Updates (Persist changes)
```

## Key Concepts

### Deterministic Execution
- Same input = same output
- No randomness or external calls
- Reproducible across nodes

### Memory Isolation
- Each contract has sandboxed memory
- No access to other contracts
- Linear memory model (0-64KB default)

### Gas Metering
- Every operation costs gas
- Prevents infinite loops
- Billing based on computation

### Contract Addressing
- Contracts have unique addresses
- Addresses are SHA-256 hashes of code
- Immutable and globally unique

## Contract Examples in Project

```bash
# View sample contracts
ls aureon-node/src/contracts/

# Available contracts:
# - hello.wat: Simple "hello" function
# - sum_amounts.wat: Add two numbers
# - mint_tokens.wat: Create new tokens
# - transfer_success.wat: Transfer validation
```

## Testing Smart Contracts

```bash
# Test WASM engine
cargo test --package aureon-node wasm::engine

# Test gas metering
cargo test --package aureon-node wasm::gas_meter

# Test contract registry
cargo test --package aureon-node contracts::registry

# Run all smart contract tests
cargo test --package aureon-node wasm::
```

## Related Examples

- `token_transfer.md` - Transfer tokens via contracts
- `spv_light_client.md` - Verify contract calls with SPV
- `production_monitoring.md` - Monitor contract execution

## References

- **WASM Engine**: `src/wasm/engine.rs`
- **Gas Metering**: `src/wasm/gas_meter.rs`
- **Host Functions**: `src/wasm/host_functions.rs`
- **Contract Registry**: `src/contracts/registry.rs`
- **Sample Contracts**: `src/contracts/`

## WebAssembly Resources

- **Official Spec**: https://webassembly.org/
- **WAT Format**: https://webassembly.github.io/spec/core/text/
- **Wasmtime Docs**: https://docs.wasmtime.dev/
