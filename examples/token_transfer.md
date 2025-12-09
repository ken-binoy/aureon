# Example: Token Transfer

Learn how to create accounts, transfer tokens, and verify balances on the Aureon blockchain.

## Overview

This example demonstrates:
1. Creating accounts with initial balances
2. Transferring tokens between accounts
3. Verifying final balances
4. Handling errors

## Code Example

```rust
use aureon_core::state::State;

fn main() -> Result<(), String> {
    // Create a new state
    let mut state = State::new();
    
    // Create two accounts
    let alice_addr = state.create_account(100.0)?;  // Alice starts with 100 tokens
    let bob_addr = state.create_account(50.0)?;      // Bob starts with 50 tokens
    
    println!("Alice address: {}", alice_addr);
    println!("Bob address: {}", bob_addr);
    
    // Check initial balances
    let alice_initial = state.get_balance(&alice_addr)?;
    let bob_initial = state.get_balance(&bob_addr)?;
    
    println!("\n--- Initial Balances ---");
    println!("Alice: {} tokens", alice_initial);
    println!("Bob: {} tokens", bob_initial);
    
    // Transfer 25 tokens from Alice to Bob
    state.transfer(&alice_addr, &bob_addr, 25.0)?;
    
    println!("\n--- After transfer (Alice -> Bob: 25 tokens) ---");
    
    // Check final balances
    let alice_final = state.get_balance(&alice_addr)?;
    let bob_final = state.get_balance(&bob_addr)?;
    
    println!("Alice: {} tokens", alice_final);
    println!("Bob: {} tokens", bob_final);
    
    // Verify
    assert_eq!(alice_final, 75.0, "Alice should have 75 tokens");
    assert_eq!(bob_final, 75.0, "Bob should have 75 tokens");
    
    println!("\n✓ Transfer successful!");
    
    Ok(())
}
```

## Running This Example

```bash
# From project root
cargo run --example token_transfer

# Expected output:
# Alice address: 0x1234...
# Bob address: 0x5678...
#
# --- Initial Balances ---
# Alice: 100 tokens
# Bob: 50 tokens
#
# --- After transfer (Alice -> Bob: 25 tokens) ---
# Alice: 75 tokens
# Bob: 75 tokens
#
# ✓ Transfer successful!
```

## Advanced: Transfer with Validation

```rust
fn safe_transfer(
    state: &mut State,
    from: &str,
    to: &str,
    amount: f64,
) -> Result<(), String> {
    // Validate amount
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    
    // Check sender has sufficient balance
    let balance = state.get_balance(from)?;
    if balance < amount {
        return Err(format!(
            "Insufficient balance: {} < {}",
            balance, amount
        ));
    }
    
    // Perform transfer
    state.transfer(from, to, amount)?;
    
    // Log transaction
    println!("Transfer: {} -> {} ({} tokens)", from, to, amount);
    
    Ok(())
}
```

## Key Concepts

### Account Creation
- Each account has a unique address
- Accounts have a balance (in tokens)
- Accounts are mutable (balances can change)

### Transfers
- Require source and destination addresses
- Amount must be positive
- Sender must have sufficient balance
- Transfers are atomic (all-or-nothing)

### Balance Verification
- Always verify balances match expectations
- Check transaction results
- Handle error cases

## Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Insufficient balance" | Account lacks funds | Check balance first |
| "Invalid address" | Address format wrong | Use correct address format |
| "Account not found" | Account doesn't exist | Create account first |
| "Negative amount" | Transfer amount < 0 | Use positive amount |

## Related Examples

- `smart_contract.md` - Execute code on balances
- `light_client.md` - Verify transfers with SPV
- `production_monitoring.md` - Monitor transfers in production

## Testing

```bash
# Test token transfers
cargo test --package aureon-core token::tests

# Run all examples
cargo test --example "*"
```

## References

- **API Documentation**: See `src/state.rs`
- **Token Module**: `src/token.rs` (balance tracking)
- **State Module**: `src/state.rs` (account management)
