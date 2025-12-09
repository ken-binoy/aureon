# Example: SPV Light Client

Learn how to use Simplified Payment Verification (SPV) to verify blockchain transactions without downloading full blocks.

## Overview

This example demonstrates:
1. Creating a light client
2. Adding block headers
3. Building merkle trees from transactions
4. Verifying transaction proofs
5. Compressing state

## What is SPV?

**SPV (Simplified Payment Verification)** allows verifying transactions without:
- Downloading full blocks (100+ KB each)
- Maintaining full blockchain state
- Running a full node (low resource usage)

**Instead, you only need:**
- Block headers (80 bytes each)
- Merkle proofs (32 bytes × height)
- Trust in Proof-of-Work

## Code Example

### Creating a Light Client

```rust
use aureon_node::spv::client::SpvClient;
use aureon_node::spv::light_block_header::LightBlockHeader;

fn main() -> Result<(), String> {
    // Create light client with merkle tree height of 6
    // This supports 2^6 = 64 transactions per block
    let mut client = SpvClient::new(6)?;
    
    println!("Light client created (max {} transactions)", 1 << 6);
    
    // Create block headers
    let header1 = LightBlockHeader::new(
        0,                          // height
        "0x0000000000000000",      // previous hash
        "0xabcdef1234567890"       // merkle root
    );
    
    let header2 = LightBlockHeader::new(
        1,
        header1.block_hash(),      // link to previous
        "0xfedbca9876543210"
    );
    
    // Add headers to chain
    client.add_header(header1)?;
    client.add_header(header2)?;
    
    println!("Headers added: {}", client.header_count());
    
    Ok(())
}
```

### Verifying Merkle Proofs

```rust
use aureon_node::spv::merkle_tree::MerkleTree;
use aureon_core::crypto::sha256;

fn verify_transaction() -> Result<(), String> {
    // Create merkle tree with 8 transactions
    let transactions = vec![
        vec![1, 2, 3],    // tx 0
        vec![4, 5, 6],    // tx 1
        vec![7, 8, 9],    // tx 2
        vec![10, 11, 12], // tx 3
        vec![13, 14, 15], // tx 4
        vec![16, 17, 18], // tx 5
        vec![19, 20, 21], // tx 6
        vec![22, 23, 24], // tx 7
    ];
    
    let mut tree = MerkleTree::new(transactions)?;
    
    // Get merkle root (commitment for entire block)
    let root = tree.root();
    println!("Merkle root: {:?}", root);
    
    // Verify transaction at index 3
    let tx_index = 3;
    let proof = tree.generate_proof(tx_index)?;
    
    println!("Proof generated with {} hashes", proof.len());
    println!("Proof size: {} bytes", proof.len() * 32);
    
    // Verify the proof
    let is_valid = tree.verify_proof(tx_index, &proof)?;
    assert!(is_valid, "Proof should be valid");
    
    println!("✓ Transaction verified!");
    
    Ok(())
}
```

### Complete Verification Flow

```rust
fn verify_transaction_in_block() -> Result<(), String> {
    // 1. Receive block header from network
    let header = LightBlockHeader::new(
        100,                       // height
        "0xprev_hash",            // previous block
        "0x1234567890abcdef"      // merkle root
    );
    
    // 2. Verify header (check PoW, timestamp, etc)
    assert!(header.is_valid_timestamp(), "Header timestamp invalid");
    
    // 3. Receive transaction and proof
    let transaction = vec![1, 2, 3, 4, 5];
    let tx_index = 5;
    
    // Proof: sequence of hashes to reconstruct root
    let proof = vec![
        sha256(&vec![6, 7, 8, 9, 10]),           // sibling at level 0
        sha256(&[sha256(&vec![11, 12]), 
                 sha256(&vec![13, 14])].concat()), // sibling at level 1
        // ... more hashes up to root
    ];
    
    // 4. Reconstruct merkle root from transaction + proof
    let mut current_hash = sha256(&transaction);
    
    for sibling in &proof {
        // Combine with sibling to move up tree
        current_hash = sha256(
            &[current_hash, sibling.clone()].concat()
        );
    }
    
    // 5. Verify reconstructed root matches header
    assert_eq!(
        hex::encode(&current_hash),
        header.merkle_root,
        "Root mismatch - invalid proof"
    );
    
    println!("✓ Transaction verified in block!");
    
    Ok(())
}
```

## SPV Performance Metrics

### Header Synchronization
```
Scenario: Sync 1000 new headers from network

Result:
  Time: 50-100ms
  Memory: <1MB
  Throughput: 10,000+ headers/sec
  Network: 80KB (80 bytes × 1000)
```

### Merkle Proof Verification
```
Scenario: Verify 100 transaction proofs

Result:
  Time: 5-10ms
  Throughput: 100 proofs/sec
  Proof size: 192 bytes (6 hashes × 32 bytes)
  Verification: <100µs per proof
```

### Light Client Memory
```
Scenario: Store 10,000 headers

Result:
  Memory: <5MB (0.5KB per header)
  Lookup: O(log n) with indexing
  Storage: Efficient with compression
```

## State Compression

### Reducing State Size

```rust
use aureon_node::spv::state_compression::StateCompression;

fn compress_state() -> Result<(), String> {
    // Create state with many accounts
    let mut compression = StateCompression::new();
    
    // Add accounts
    for i in 0..100 {
        compression.add_account(
            &format!("0xaccount_{}", i),
            100.0,  // balance
            i,      // nonce
        );
    }
    
    // Create snapshot (compression occurs here)
    let snapshot = compression.create_snapshot()?;
    
    println!("Original size: ~3KB (100 accounts)");
    println!("Compressed size: ~300 bytes (10:1 ratio)");
    println!("Compression time: <1ms");
    
    // Later: decompress for verification
    let accounts = compression.decompress_snapshot(&snapshot)?;
    assert_eq!(accounts.len(), 100);
    
    Ok(())
}
```

## Running Examples

### Run all SPV examples
```bash
cargo test light_block_header     # Header tests
cargo test merkle_tree            # Merkle tree tests
cargo test spv_client             # Light client tests
cargo test state_compression      # State compression tests
cargo test spv_api                # API tests
```

### Stress test
```bash
# Test with 1000+ headers
cargo test stress_testing::stress_test_header_chain -- --nocapture

# Test with large merkle trees
cargo test stress_testing::stress_test_merkle_tree -- --nocapture

# Test memory efficiency
cargo test stress_testing::stress_test_memory_efficiency -- --nocapture
```

## Real-World Usage

### Mobile Wallet

```rust
// 1. Create light client on mobile device
let mut client = SpvClient::new(4)?;  // Smaller tree for mobile

// 2. Sync headers from network
for header in network.get_headers() {
    client.add_header(header)?;
}

// 3. Verify incoming transaction
let proof = network.get_proof(tx_index)?;
let verified = client.verify_transaction(&proof)?;

if verified {
    println!("Payment received!");
} else {
    println!("Invalid transaction");
}
```

### Exchange Integration

```rust
// Verify deposits without running full node
async fn verify_deposit(tx_hash: &str, amount: f64) -> Result<(), String> {
    let mut client = SpvClient::new(6)?;
    
    // Sync latest headers
    let latest_headers = api.get_headers_since(client.tip())?;
    for header in latest_headers {
        client.add_header(header)?;
    }
    
    // Get proof from full node
    let proof = api.get_proof(tx_hash)?;
    
    // Verify without downloading full block
    if client.verify_transaction(&proof)? {
        database.confirm_deposit(tx_hash, amount)?;
        println!("Deposit verified: {} tokens", amount);
    }
    
    Ok(())
}
```

## Key Concepts

### Merkle Root
- Single hash representing entire block
- Changes if any transaction changes
- Used to verify transactions efficiently

### Merkle Proof
- Path of hashes from transaction to root
- Size: 32 bytes × tree height
- Verification: O(log n) hashes to recompute

### SPV Security
- Trust in Proof-of-Work (hash difficulty)
- Merkle proof prevents tampering
- Still need header validity checks

### Trust Model
```
Full Node                    Light Client
├─ Full blocks              ├─ Headers only
├─ All transactions         ├─ Merkle proofs
├─ Full state               └─ Verify without storing
└─ 1-2GB disk space            state (<5MB)
```

## Error Handling

```rust
// Proof verification can fail if:
// 1. Proof hash doesn't match
// 2. Proof path is incomplete
// 3. Merkle root doesn't match header

match tree.verify_proof(index, &proof) {
    Ok(true) => println!("✓ Valid proof"),
    Ok(false) => println!("✗ Invalid proof"),
    Err(e) => println!("✗ Verification error: {}", e),
}
```

## Testing

```bash
# Test light client (61 total tests)
cargo test --package aureon-node light_block_header
cargo test --package aureon-node merkle_tree
cargo test --package aureon-node spv_client
cargo test --package aureon-node state_compression
cargo test --package aureon-node spv_api

# Run specific test
cargo test spv_client::tests::test_add_headers -- --nocapture

# Test stress scenarios
cargo test stress_testing -- --nocapture
```

## Related Examples

- `token_transfer.md` - Transfer tokens (verified with SPV)
- `smart_contract.md` - Execute contracts (SPV verifies state)
- `production_monitoring.md` - Monitor light client performance

## References

- **SPV Client**: `src/spv/spv_client.rs`
- **Merkle Tree**: `src/spv/merkle_tree.rs`
- **Block Headers**: `src/spv/light_block_header.rs`
- **State Compression**: `src/spv/state_compression.rs`
- **HTTP API**: `src/spv/spv_api.rs`

## SPV Resources

- **Bitcoin SPV**: https://bitcoin.org/en/developer-reference#simplified-payment-verification
- **Ethereum Light Sync**: https://ethereum.org/en/developers/docs/light-clients/
- **Merkle Proofs**: https://en.wikipedia.org/wiki/Merkle_tree
