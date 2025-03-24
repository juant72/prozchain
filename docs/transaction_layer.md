# Transaction Layer Documentation

## 1. Overview
The Transaction Layer manages creation, propagation, validation, and execution of operations that change the blockchain state. Its goal is to ensure integrity and proper ordering.

## 2. Transaction Structure

### 2.1 Basic Transaction Format
Every transaction has key fields (sender, receiver, amount, data, gas fees).  
*Explanation:* It’s like a payment order: specifies who sends, who receives, and how much.
```rust
struct Transaction {
    version: u16,
    nonce: u64,
    chain_id: u64,
    from: Address,
    to: Option<Address>,
    value: Amount,
    data: Vec<u8>,
    gas_limit: u64,
    gas_price: Amount,
    public_key: PublicKey,
    signature: Signature,
    expiry_height: Option<BlockHeight>,
    dependencies: Vec<TransactionId>,
    priority_fee: Option<Amount>,
}
```

### 2.2 Transaction Identifiers
Computed using a BLAKE3 hash of the unsigned transaction to uniquely identify it.
```rust
fn compute_transaction_id(tx: &Transaction) -> TransactionId {
    let encoded = encode_unsigned_transaction(tx); // ...existing code...
    blake3::hash(&encoded).into()
}
```
*Explanation:* Prevents duplicate transactions from being processed.

### 2.3 Transaction Types
Differentiates types: transfers, smart contract deployments, contract calls, etc.

## 3. Transaction Lifecycle

### 3.1 Creation and Signing
Users create and sign transactions for authenticity.
```rust
fn create_signed_transaction(
    sender: &Keypair, to: Option<Address>, value: Amount, data: Vec<u8>,
    nonce: u64, gas_limit: u64, gas_price: Amount, chain_id: u64
) -> Result<Transaction> {
    // ...existing code...
    // Build, sign, and return the transaction.
}
```
*Explanation:* Deterministic signing prevents replay attacks.

### 3.2 Validation Rules
Checks structure, sufficient balance, correct nonce, and gas limits.
```rust
fn validate_transaction(tx: &Transaction, state: &State) -> Result<ValidationOutcome> {
    // ...existing code...
}
```

### 3.3 Mempool Management
The mempool queues pending transactions, prioritizing them by fee and waiting time.
```rust
struct Mempool {
    transactions: BTreeMap<TransactionPriority, HashSet<TransactionId>>,
    by_id: HashMap<TransactionId, Transaction>,
    by_sender: HashMap<Address, BTreeSet<(Nonce, TransactionId)>>,
    capacity: usize,
    min_fee_per_gas: Amount,
}
```
*Explanation:* Allows lower-fee transactions to be replaced and expired ones to be pruned.

## 4. Fee Market and Prioritization

### 4.1 Fee Structure
Comprises an adjustable base fee and a priority fee.
```rust
fn calculate_next_base_fee(current_base: Amount, target: Gas, used: Gas) -> Amount {
    // ...existing code...
}
```
*Explanation:* Adjusts the base fee to balance resource demand.

### 4.2 Transaction Priority Calculation
Combines effective gas fee and mempool waiting time.
```rust
fn calculate_transaction_priority(tx: &Transaction, base_fee: Amount, pool_time: Duration) -> TransactionPriority {
    // ...existing code...
}
```

## 5. Transaction Execution

### 5.1 Execution Environment
Creates an execution context with block and transaction data.
```rust
struct ExecutionContext {
    tx: Transaction,
    sender: Address,
    recipient: Option<Address>,
    value: Amount,
    data: Bytes,
    gas_limit: Gas,
    gas_price: Amount,
    block_number: BlockNumber,
    block_timestamp: Timestamp,
    chain_id: ChainId,
    state: State,
}
```

### 5.2 Execution Stages
Includes pre-execution (deduct balance, increment nonce), execution, and post-execution (refund, logging).
```rust
fn execute_transaction(tx: &Transaction, state: &mut State, ctx: &BlockContext) -> ExecutionResult {
    // ...existing code...
}
```

### 5.3 Parallel Transaction Execution
Groups non-conflicting transactions using a dependency graph for parallel execution.
```rust
fn parallel_execute_transactions(txs: &[Transaction], state: &State) -> Vec<ExecutionResult> {
    // ...existing code...
}
```

## 6. Transaction Receipts
Records the execution outcome (index, gas used, logs, etc.).
```rust
struct TransactionReceipt {
    transaction_id: TransactionId,
    transaction_index: u32,
    block_number: BlockNumber,
    block_hash: BlockHash,
    from: Address,
    to: Option<Address>,
    contract_address: Option<Address>,
    gas_used: Gas,
    gas_limit: Gas,
    status: ExecutionStatus,
    logs: Vec<Log>,
    logs_bloom: BloomFilter,
    state_root: Option<Hash>,
}
```

## 7. Special Transactions
Formats for delegated or batch transactions.
```rust
struct DelegatedTransaction { /*...existing code...*/ }
struct BatchTransaction { /*...existing code...*/ }
```

## 8. Transaction API
Functions for submitting transactions and querying their status.
```rust
fn submit_transaction(tx: Transaction) -> Result<TransactionId> {
    // ...existing code...
}
fn get_transaction_status(tx_id: TransactionId) -> TransactionStatus {
    // ...existing code...
}
```

## 9. Transaction Indexing and Querying
Indexes for efficient lookup by ID, block, address, and time range.
```rust
struct TransactionIndex {
    by_id: HashMap<TransactionId, TransactionMetadata>,
    by_block: HashMap<BlockNumber, Vec<TransactionId>>,
    by_address: HashMap<Address, Vec<TransactionId>>,
    by_timestamp_range: BTreeMap<Timestamp, Vec<TransactionId>>,
}
```

## 10. Security Considerations
Protection against replay (nonce, chain-id) and mitigation of front-running.

## 11. Optimizations
Advanced mempool management, batch signature verification, and parallel validation.

## 12. Future Improvements
Plans for account abstraction, meta‑transactions, and cross‑chain protocols.

## 13. References
Includes references to EIPs, academic papers, and key specifications.
