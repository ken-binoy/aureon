# API Reference Documentation

Complete API reference for Aureon blockchain modules and interfaces.

## Table of Contents

1. [Core Types](#core-types)
2. [State Management](#state-management)
3. [Consensus](#consensus)
4. [Smart Contracts](#smart-contracts)
5. [Light Client (SPV)](#light-client-spv)
6. [Production Hardening](#production-hardening)
7. [HTTP REST API](#http-rest-api)

## Core Types

### Transaction

```rust
pub struct Transaction {
    pub from: String,          // Sender address
    pub to: String,            // Recipient address
    pub amount: f64,           // Amount to transfer
    pub nonce: u64,            // Sender's transaction count
    pub signature: Vec<u8>,    // Transaction signature
}

impl Transaction {
    /// Create a new transaction
    pub fn new(from: String, to: String, amount: f64, nonce: u64) -> Self
    
    /// Sign transaction with private key
    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), String>
    
    /// Verify transaction signature
    pub fn verify(&self) -> Result<bool, String>
    
    /// Get transaction hash
    pub fn hash(&self) -> String
}
```

### Block

```rust
pub struct Block {
    pub height: u32,                  // Block number
    pub timestamp: u64,               // Block creation time
    pub previous_hash: String,        // Hash of previous block
    pub state_root: String,           // Root hash of state tree
    pub transactions: Vec<Transaction>, // Included transactions
    pub validator: String,            // Block proposer
}

impl Block {
    /// Create new block
    pub fn new(
        height: u32,
        previous_hash: String,
        state_root: String,
        validator: String,
    ) -> Self
    
    /// Calculate block hash
    pub fn hash(&self) -> String
    
    /// Validate block structure
    pub fn validate(&self) -> Result<(), String>
    
    /// Get transaction count
    pub fn transaction_count(&self) -> usize
}
```

### Account

```rust
pub struct Account {
    pub address: String,       // Account identifier
    pub balance: f64,          // Token balance
    pub nonce: u64,           // Transaction count
    pub code: Vec<u8>,        // Contract code (if any)
}

impl Account {
    /// Get account balance
    pub fn balance(&self) -> f64
    
    /// Get next transaction nonce
    pub fn next_nonce(&self) -> u64
    
    /// Increment nonce
    pub fn increment_nonce(&mut self)
    
    /// Has code (is contract)
    pub fn is_contract(&self) -> bool
}
```

## State Management

### State Trie

```rust
pub struct State {
    accounts: HashMap<String, Account>,
    root: String,
}

impl State {
    /// Create new empty state
    pub fn new() -> Self
    
    /// Create account with initial balance
    pub fn create_account(&mut self, address: String, balance: f64)
        -> Result<(), String>
    
    /// Get account by address
    pub fn get_account(&self, address: &str) -> Option<&Account>
    
    /// Get account balance
    pub fn get_balance(&self, address: &str) -> Result<f64, String>
    
    /// Transfer tokens between accounts
    pub fn transfer(&mut self, from: &str, to: &str, amount: f64)
        -> Result<(), String>
    
    /// Update account nonce
    pub fn increment_nonce(&mut self, address: &str)
        -> Result<(), String>
    
    /// Get state root hash
    pub fn root(&self) -> String
    
    /// Apply transaction to state
    pub fn apply_transaction(&mut self, tx: &Transaction)
        -> Result<(), String>
}
```

### Merkle Patricia Trie

```rust
pub struct MerklePatriciaTrie {
    root: TrieNode,
}

impl MerklePatriciaTrie {
    /// Create new empty trie
    pub fn new() -> Self
    
    /// Insert key-value pair
    pub fn insert(&mut self, key: String, value: Vec<u8>)
    
    /// Get value by key
    pub fn get(&self, key: &str) -> Option<Vec<u8>>
    
    /// Delete key
    pub fn delete(&mut self, key: &str) -> bool
    
    /// Get root hash
    pub fn root_hash(&self) -> String
    
    /// Generate merkle proof for key
    pub fn generate_proof(&self, key: &str) -> Result<Vec<Vec<u8>>, String>
    
    /// Verify merkle proof
    pub fn verify_proof(&self, key: &str, proof: &[Vec<u8>])
        -> Result<bool, String>
}
```

## Consensus

### Proof of Stake

```rust
pub struct ProofOfStake {
    validators: HashMap<String, Validator>,
    total_stake: f64,
}

pub struct Validator {
    pub address: String,
    pub stake: f64,
    pub commission: f64,
    pub delegators: HashMap<String, f64>,
}

impl ProofOfStake {
    /// Create new PoS consensus
    pub fn new() -> Self
    
    /// Register new validator
    pub fn register_validator(
        &mut self,
        address: String,
        stake: f64,
        commission: f64,
    ) -> Result<(), String>
    
    /// Delegate tokens to validator
    pub fn delegate(
        &mut self,
        delegator: String,
        validator: String,
        amount: f64,
    ) -> Result<(), String>
    
    /// Select block proposer
    pub fn select_proposer(&self) -> Result<String, String>
    
    /// Distribute block reward
    pub fn distribute_reward(
        &mut self,
        validator: String,
        reward: f64,
    ) -> Result<(), String>
    
    /// Slash validator
    pub fn slash_validator(
        &mut self,
        validator: String,
        penalty_percent: f64,
    ) -> Result<(), String>
}
```

## Smart Contracts

### WASM Engine

```rust
pub struct WasmEngine {
    // Internal state
}

impl WasmEngine {
    /// Create new WASM engine
    pub fn new() -> Self
    
    /// Execute contract function
    pub fn execute(
        &self,
        code: &[u8],
        function: &str,
        args: &[i32],
    ) -> Result<i32, String>
    
    /// Validate WASM bytecode
    pub fn validate(&self, code: &[u8]) -> Result<(), String>
    
    /// Get function signature
    pub fn get_signature(
        &self,
        code: &[u8],
        function: &str,
    ) -> Result<FunctionSignature, String>
}

pub struct FunctionSignature {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}
```

### Gas Meter

```rust
pub struct GasMeter {
    total_cost: u64,
    max_cost: u64,
}

impl GasMeter {
    /// Create new gas meter
    pub fn new(max_cost: u64) -> Self
    
    /// Get current gas cost
    pub fn cost(&self) -> u64
    
    /// Get remaining gas
    pub fn remaining(&self) -> u64
    
    /// Check if over budget
    pub fn is_over_limit(&self) -> bool
    
    /// Record operation cost
    pub fn record_operation(&mut self, cost: u64) -> Result<(), String>
}

// Gas costs
pub const GAS_PER_OPERATION: u64 = 1_000;
pub const GAS_MEMORY_GROW: u64 = 10_000;
pub const GAS_MEMORY_LOAD: u64 = 50;
pub const GAS_MEMORY_STORE: u64 = 100;
```

### Contract Registry

```rust
pub struct ContractRegistry {
    contracts: HashMap<String, (String, Vec<u8>)>,
}

impl ContractRegistry {
    /// Create new registry
    pub fn new() -> Self
    
    /// Deploy contract and get address
    pub fn deploy(&mut self, code: Vec<u8>) -> Result<String, String>
    
    /// Get contract code by address
    pub fn get_contract(&self, address: &str) -> Option<Vec<u8>>
    
    /// Check if contract exists
    pub fn contract_exists(&self, address: &str) -> bool
    
    /// Get contract address from code
    pub fn address_for_code(&self, code: &[u8]) -> String
}
```

## Light Client (SPV)

### SPV Client

```rust
pub struct SpvClient {
    headers: Vec<LightBlockHeader>,
    merkle_height: usize,
}

impl SpvClient {
    /// Create new light client
    pub fn new(merkle_height: usize) -> Result<Self, String>
    
    /// Add block header to chain
    pub fn add_header(&mut self, header: LightBlockHeader)
        -> Result<(), String>
    
    /// Get current tip header
    pub fn tip(&self) -> Option<&LightBlockHeader>
    
    /// Get header at height
    pub fn header_at(&self, height: u32) -> Option<&LightBlockHeader>
    
    /// Get total headers
    pub fn header_count(&self) -> usize
    
    /// Verify transaction in block
    pub fn verify_transaction(&self, tx_hash: &str, proof: &[Vec<u8>])
        -> Result<bool, String>
}
```

### Light Block Header

```rust
pub struct LightBlockHeader {
    pub height: u32,
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
}

impl LightBlockHeader {
    /// Create new header
    pub fn new(height: u32, previous_hash: String, merkle_root: String) -> Self
    
    /// Calculate header hash
    pub fn block_hash(&self) -> String
    
    /// Validate header
    pub fn validate(&self) -> Result<(), String>
    
    /// Check if valid timestamp
    pub fn is_valid_timestamp(&self) -> bool
}
```

### Merkle Tree

```rust
pub struct MerkleTree {
    transactions: Vec<Vec<u8>>,
    tree: Vec<Vec<Vec<u8>>>,
}

impl MerkleTree {
    /// Create merkle tree from transactions
    pub fn new(transactions: Vec<Vec<u8>>) -> Result<Self, String>
    
    /// Get merkle root
    pub fn root(&self) -> Vec<u8>
    
    /// Generate proof for transaction
    pub fn generate_proof(&self, index: usize)
        -> Result<Vec<Vec<u8>>, String>
    
    /// Verify proof
    pub fn verify_proof(
        &self,
        index: usize,
        proof: &[Vec<u8>],
    ) -> Result<bool, String>
    
    /// Get proof size
    pub fn proof_size(&self, index: usize) -> usize
}
```

### State Compression

```rust
pub struct StateCompression {
    snapshots: HashMap<u32, Vec<u8>>,
    accounts: HashMap<String, Account>,
}

impl StateCompression {
    /// Create new state compression
    pub fn new() -> Self
    
    /// Add account to current snapshot
    pub fn add_account(
        &mut self,
        address: String,
        balance: f64,
        nonce: u64,
    )
    
    /// Create compressed snapshot
    pub fn create_snapshot(&mut self) -> Result<Vec<u8>, String>
    
    /// Decompress snapshot
    pub fn decompress_snapshot(&mut self, snapshot: &[u8])
        -> Result<Vec<Account>, String>
    
    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64
}
```

## Production Hardening

### Error Recovery

```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
}

pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    /// Check if request allowed
    pub fn allow_request(&mut self) -> bool
    
    /// Record successful operation
    pub fn record_success(&mut self)
    
    /// Record failed operation
    pub fn record_failure(&mut self)
    
    /// Get current state
    pub fn state(&self) -> CircuitState
}

pub struct RateLimiter {
    capacity: u32,
    tokens: u32,
    refill_rate: u32,
}

impl RateLimiter {
    /// Try to acquire token
    pub fn try_acquire(&mut self) -> bool
    
    /// Get available tokens
    pub fn available_tokens(&self) -> u32
    
    /// Refill tokens
    pub fn refill(&mut self)
}
```

### Performance Optimization

```rust
pub struct LruCache<K, V> {
    data: HashMap<K, V>,
    access_order: VecDeque<K>,
    max_size: usize,
}

impl<K: Clone + Eq + Hash, V: Clone> LruCache<K, V> {
    /// Create new cache
    pub fn new(max_size: usize) -> Self
    
    /// Get value from cache
    pub fn get(&mut self, key: &K) -> Option<V>
    
    /// Insert value
    pub fn insert(&mut self, key: K, value: V)
    
    /// Get cache size
    pub fn len(&self) -> usize
    
    /// Get hit rate
    pub fn hit_rate(&self) -> f64
}

pub struct Lazy<T> {
    value: Option<T>,
    computed: bool,
}

impl<T> Lazy<T> {
    /// Create new lazy value
    pub fn new() -> Self
    
    /// Get or compute value
    pub fn get_or_compute<F: FnOnce() -> T>(&mut self, f: F) -> &T
    
    /// Reset computed value
    pub fn reset(&mut self)
}
```

### Monitoring

```rust
pub struct LatencyTracker {
    name: String,
    latencies: Vec<u64>,
    max_samples: usize,
}

impl LatencyTracker {
    /// Create new tracker
    pub fn new(name: &str) -> Self
    
    /// Record latency in microseconds
    pub fn record_latency_us(&mut self, latency_us: u64)
    
    /// Get average latency
    pub fn average_latency(&self) -> u64
    
    /// Get p95 latency
    pub fn p95_latency(&self) -> u64
    
    /// Get p99 latency
    pub fn p99_latency(&self) -> u64
    
    /// Get sample count
    pub fn sample_count(&self) -> usize
}

pub struct ErrorRateTracker {
    total_operations: u32,
    total_errors: u32,
    error_types: HashMap<String, u32>,
}

impl ErrorRateTracker {
    /// Create new tracker
    pub fn new() -> Self
    
    /// Record operation
    pub fn record_operation(&mut self)
    
    /// Record error
    pub fn record_error(&mut self, error_type: &str)
    
    /// Get error rate
    pub fn error_rate(&self) -> f64
    
    /// Get most common error
    pub fn most_common_error(&self) -> Option<String>
}

pub struct HealthDashboard {
    service_name: String,
    status: HealthStatus,
    latencies: HashMap<String, LatencyTracker>,
    error_tracker: ErrorRateTracker,
}

impl HealthDashboard {
    /// Create new dashboard
    pub fn new(service_name: &str) -> Self
    
    /// Record operation latency
    pub fn record_latency(&mut self, operation: &str, latency_us: u64)
    
    /// Record operation error
    pub fn record_error(&mut self, operation: &str, error_type: &str)
    
    /// Record operation
    pub fn record_operation(&mut self, operation: &str)
    
    /// Update health status
    pub fn update_health_status(&mut self)
    
    /// Generate health report
    pub fn generate_report(&self) -> String
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

## HTTP REST API

### GET Endpoints

```
GET /chain/head
  Response: { height: u32, hash: string, state_root: string }
  
GET /block/:height
  Response: Block
  
GET /balance/:address
  Response: { balance: f64 }
  
GET /nonce/:address
  Response: { nonce: u64 }
  
GET /peers
  Response: [Peer]
  
GET /validators
  Response: [Validator]
```

### POST Endpoints

```
POST /submit-tx
  Request: Transaction
  Response: { hash: string, status: "pending" | "accepted" }
  
POST /call-contract
  Request: { address: string, function: string, args: [i32] }
  Response: { result: i32, gas_used: u64 }
  
POST /deploy-contract
  Request: { bytecode: Vec<u8> }
  Response: { address: string }
```

### WebSocket Endpoints

```
WS /chain/events
  Publishes: BlockProposed, TransactionIncluded, Error
  
WS /peer-events
  Publishes: PeerConnected, PeerDisconnected, PeerSynced
```

## Constants

```rust
// Consensus
pub const MINIMUM_STAKE: f64 = 32.0;
pub const MAX_VALIDATORS: usize = 100;
pub const BLOCK_REWARD: f64 = 5.0;
pub const SLASHING_PENALTY: f64 = 0.1;

// Gas
pub const GAS_PER_OPERATION: u64 = 1_000;
pub const MAX_GAS_PER_CONTRACT: u64 = 100_000_000;

// Network
pub const DEFAULT_P2P_PORT: u16 = 6000;
pub const DEFAULT_API_PORT: u16 = 8080;
pub const MAX_PEERS: usize = 50;

// SPV
pub const DEFAULT_MERKLE_HEIGHT: usize = 6;
pub const PROOF_SIZE_PER_LEVEL: usize = 32;

// Monitoring
pub const LATENCY_SAMPLE_SIZE: usize = 1000;
pub const ERROR_RATE_THRESHOLD: f64 = 0.05;
pub const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;
```

## Error Types

```rust
pub enum AureonError {
    // State errors
    AccountNotFound(String),
    InsufficientBalance,
    InvalidNonce,
    
    // Consensus errors
    InvalidBlockHash,
    InvalidSignature,
    UnknownValidator,
    
    // Contract errors
    ContractNotFound(String),
    ExecutionFailed(String),
    OutOfGas,
    
    // Network errors
    PeerNotFound,
    ConnectionFailed,
    SyncFailed,
    
    // SPV errors
    InvalidProof,
    HeaderNotFound,
    StateCompressionFailed,
}

impl std::fmt::Display for AureonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Error formatting
    }
}
```

---

**API Version**: 1.0.0
**Last Updated**: December 2025
**Stability**: Stable ✅
