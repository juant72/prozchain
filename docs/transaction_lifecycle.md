# Transaction Lifecycle Documentation

## 1. Overview
This document explains the complete journey of a transaction through the ProzChain system, from creation to final inclusion in the blockchain state. Understanding this lifecycle is essential for developers building on the platform and for troubleshooting transaction issues.

**Why This Matters**: Transactions are the fundamental unit of work in a blockchain. Every change to the state - whether transferring tokens, deploying contracts, or calling contract functions - happens through transactions.

## 2. Transaction Creation and Signing

### 2.1 Creating the Transaction
A transaction begins with the formulation of its intent and parameters.

```rust
struct TransactionRequest {
    nonce: Option<u64>,         // Optional, can be auto-filled
    to: Option<Address>,        // None for contract creation
    value: Amount,              // Token amount to transfer
    data: Vec<u8>,              // Call data or contract initialization code
    gas_limit: Option<u64>,     // Optional, can be estimated
    gas_price: Option<Amount>,  // Optional, can use market rate
    priority_fee: Option<Amount>, // Optional tip for validators
}

fn create_transaction(
    request: TransactionRequest, 
    chain_id: u64, 
    account_service: &AccountService
) -> Result<Transaction> {
    // Fill in missing nonce if needed
    let nonce = match request.nonce {
        Some(n) => n,
        None => account_service.get_next_nonce(&tx_context.sender)?,
    };
    
    // Estimate gas if needed
    let gas_limit = match request.gas_limit {
        Some(limit) => limit,
        None => estimate_gas(&request, account_service)?,
    };
    
    // Determine gas price based on current market
    let gas_price = match request.gas_price {
        Some(price) => price,
        None => account_service.get_suggested_gas_price()?,
    };
    
    // Set priority fee
    let priority_fee = request.priority_fee.unwrap_or_default();
    
    // Build the complete transaction
    let transaction = Transaction {
        version: CURRENT_TX_VERSION,
        nonce,
        chain_id,
        to: request.to,
        value: request.value,
        data: request.data,
        gas_limit,
        gas_price,
        priority_fee,
        // Signature fields will be filled later
        // ...existing code...
    };
    
    Ok(transaction)
}
```

**Design Rationale**:
- **Convenience**: Optional fields can be auto-filled for simpler API usage
- **Flexibility**: Supports different transaction types (transfer, contract deployment, contract call)
- **Smart Defaults**: Uses network-appropriate values for gas price and limits
- **Chain ID**: Prevents replay attacks across different networks

**For Beginners**: This is like filling out a form with your payment details and what you want to purchase, but some fields can be automatically filled in for convenience.

### 2.2 Signing the Transaction
The transaction must be cryptographically signed to prove it's authorized by the sender.

```rust
fn sign_transaction(transaction: &mut Transaction, private_key: &PrivateKey) -> Result<()> {
    // Create a digest of the transaction fields for signing
    let message = hash_transaction_for_signing(transaction);
    
    // Sign the digest with the private key
    let signature = match private_key.key_type() {
        KeyType::Ed25519 => sign_ed25519(private_key, &message)?,
        KeyType::Secp256k1 => sign_secp256k1(private_key, &message)?,
        // Other signature schemes
        // ...existing code...
    };
    
    // Add signature and public key to the transaction
    transaction.signature = signature;
    transaction.public_key = private_key.public_key();
    
    Ok(())
}
```

**Design Rationale**:
- **Multiple Signature Schemes**: Supports different cryptographic algorithms
- **Standard Structure**: Consistent signing process across all transaction types
- **Replay Protection**: Includes nonce and chain ID in the signed message
- **Validation Ready**: Includes public key for easy verification

**For Beginners**: This is like using your private signature to authorize a check or contract, proving you're the one who authorized the transaction.

## 3. Transaction Submission and Validation

### 3.1 Submitting to the Network
The signed transaction is sent to the blockchain network for processing.

```rust
async fn submit_transaction(
    transaction: &SignedTransaction,
    network_client: &NetworkClient
) -> Result<TransactionHash> {
    // Perform initial validation
    validate_transaction_format(transaction)?;
    
    // Encode transaction for network transmission
    let encoded = encode_transaction(transaction)?;
    
    // Send to network
    let response = network_client
        .post_transaction(encoded)
        .await?;
    
    // Parse response
    let tx_hash = response.transaction_hash();
    
    Ok(tx_hash)
}
```

**Design Rationale**:
- **Basic Validation**: Early checking before network resources are used
- **Efficient Encoding**: Compact binary representation for network transmission
- **Asynchronous**: Non-blocking operation for responsive user interfaces
- **Immediate Feedback**: Returns transaction hash for future status checking

**For Beginners**: This is like dropping a letter in the mailbox - you're submitting the transaction to the network for delivery, and getting a tracking number (the transaction hash) in return.

### 3.2 Mempool Validation
Before a transaction enters the mempool, it undergoes comprehensive validation.

```rust
fn validate_for_mempool(
    transaction: &SignedTransaction,
    state: &State,
    mempool: &Mempool
) -> Result<ValidationOutcome> {
    // Verify signature
    if !verify_transaction_signature(transaction)? {
        return Err(Error::InvalidSignature);
    }
    
    // Check if chain ID matches
    if transaction.chain_id != get_network_chain_id() {
        return Err(Error::ChainIdMismatch);
    }
    
    // Verify nonce is correct
    let expected_nonce = state.get_nonce(&transaction.from())?;
    if transaction.nonce < expected_nonce {
        return Err(Error::NonceTooLow);
    }
    if transaction.nonce > expected_nonce + MAX_NONCE_AHEAD {
        return Err(Error::NonceTooHigh);
    }
    
    // Check sufficient balance for gas + value
    let balance = state.get_balance(&transaction.from())?;
    let required_balance = transaction.value + (transaction.gas_limit * transaction.gas_price);
    if balance < required_balance {
        return Err(Error::InsufficientBalance);
    }
    
    // Check gas limit is sufficient for base operation
    let min_gas = calculate_minimum_gas(transaction)?;
    if transaction.gas_limit < min_gas {
        return Err(Error::GasLimitTooLow);
    }
    
    // Check if transaction already exists in mempool
    if mempool.contains(&transaction.hash())? {
        return Err(Error::AlreadyInMempool);
    }
    
    // Additional validity checks
    // ...existing code...
    
    Ok(ValidationOutcome::Valid)
}
```

**Design Rationale**:
- **Comprehensive Checks**: Validates all aspects of transaction validity
- **Early Rejection**: Prevents invalid transactions from consuming resources
- **Clear Error Messages**: Helps developers understand validation failures
- **DoS Protection**: Limits resource usage with reasonable bounds

**For Beginners**: This is like quality control at a mail sorting facility - checking that your letter has proper postage, a valid address, and meets all the requirements before it's accepted for delivery.

### 3.3 Mempool Management
Valid transactions are stored in the mempool until they're included in a block.

```rust
struct Mempool {
    transactions: BTreeMap<TransactionPriority, HashSet<TransactionHash>>,
    by_hash: HashMap<TransactionHash, SignedTransaction>,
    by_sender: HashMap<Address, BTreeMap<Nonce, TransactionHash>>,
    capacity: usize,
}

impl Mempool {
    fn add_transaction(&mut self, tx: SignedTransaction) -> Result<()> {
        // Check if mempool is full
        if self.size() >= self.capacity && !self.would_replace_existing(&tx) {
            return Err(Error::MempoolFull);
        }
        
        let hash = tx.hash();
        let sender = tx.from();
        let nonce = tx.nonce;
        let priority = calculate_transaction_priority(&tx);
        
        // Replace any existing transaction with same sender and nonce
        if let Some(existing) = self.by_sender
            .get(&sender)
            .and_then(|txs| txs.get(&nonce))
        {
            let existing_tx = self.by_hash.get(existing).unwrap();
            let existing_priority = calculate_transaction_priority(existing_tx);
            
            // Only replace if new transaction has higher priority
            if priority <= existing_priority {
                return Err(Error::ReplacementUnderpriced);
            }
            
            // Remove existing transaction
            self.remove_transaction(existing);
        }
        
        // Add new transaction
        self.transactions
            .entry(priority)
            .or_default()
            .insert(hash);
            
        self.by_hash.insert(hash, tx.clone());
        
        self.by_sender
            .entry(sender)
            .or_default()
            .insert(nonce, hash);
        
        Ok(())
    }
    
    fn get_transactions_for_block(&self, limit: usize) -> Vec<SignedTransaction> {
        // Start with highest priority transactions
        let mut result = Vec::with_capacity(limit);
        let mut account_nonces: HashMap<Address, Nonce> = HashMap::new();
        
        for (_, tx_hashes) in self.transactions.iter().rev() {
            for hash in tx_hashes {
                let tx = self.by_hash.get(hash).unwrap();
                let sender = tx.from();
                let nonce = tx.nonce;
                
                // Only include transactions with sequential nonces
                let next_nonce = account_nonces.get(&sender).copied().unwrap_or_default();
                if nonce != next_nonce {
                    continue;
                }
                
                result.push(tx.clone());
                account_nonces.insert(sender, nonce + 1);
                
                if result.len() >= limit {
                    return result;
                }
            }
        }
        
        result
    }
}
```

**Design Rationale**:
- **Priority Ordering**: Orders transactions by fee to maximize validator revenue
- **Nonce Tracking**: Ensures transactions are ordered correctly for each sender
- **Replacement Rules**: Allows higher-fee transactions to replace lower-fee ones
- **Capacity Management**: Prevents memory exhaustion with size limits

**For Beginners**: The mempool is like a waiting room where transactions sit until they're selected for inclusion in a block, with higher-paying transactions generally getting priority treatment.

## 4. Block Inclusion and Execution

### 4.1 Block Selection
A block producer selects transactions from the mempool to include in a new block.

```rust
fn select_transactions_for_block(
    mempool: &Mempool, 
    state: &State, 
    block_gas_limit: Gas
) -> Vec<SignedTransaction> {
    let mut selected_txs = Vec::new();
    let mut gas_used = 0;
    let mut account_nonces: HashMap<Address, Nonce> = HashMap::new();
    
    // Start with highest priority transactions
    let candidates = mempool.get_transactions_by_priority();
    
    for tx in candidates {
        // Check if we've hit the gas limit
        let tx_gas = tx.gas_limit;
        if gas_used + tx_gas > block_gas_limit {
            continue;  // Skip this transaction, try next one
        }
        
        let sender = tx.from();
        let expected_nonce = account_nonces
            .get(&sender)
            .copied()
            .unwrap_or_else(|| state.get_nonce(&sender).unwrap_or_default());
            
        // Ensure transactions are in nonce order
        if tx.nonce != expected_nonce {
            continue;  // Skip, nonce doesn't match expected next nonce
        }
        
        // Add transaction to selected list
        selected_txs.push(tx);
        gas_used += tx_gas;
        account_nonces.insert(sender, expected_nonce + 1);
        
        // Check if block is full
        if gas_used >= TARGET_BLOCK_GAS_USAGE {
            break;
        }
    }
    
    selected_txs
}
```

**Design Rationale**:
- **Economic Optimization**: Prioritizes transactions with higher fees
- **Gas Limit**: Ensures block doesn't exceed resource constraints
- **Nonce Ordering**: Maintains transaction ordering requirements
- **Greedy Algorithm**: Simple but effective transaction selection

**For Beginners**: This is like a bus driver deciding which passengers to pick up based on how much they're willing to pay and how much space is left on the bus, while making sure people from the same family board in the correct order.

### 4.2 Transaction Execution
Each selected transaction is executed against the blockchain state.

```rust
fn execute_transaction(
    tx: &SignedTransaction,
    state: &mut State,
    env: &ExecutionEnvironment
) -> Result<TransactionReceipt> {
    // Create execution context
    let mut context = ExecutionContext {
        block_height: env.block_height,
        block_timestamp: env.block_timestamp,
        gas_price: tx.gas_price,
        gas_limit: tx.gas_limit,
        origin: tx.from(),
    };
    
    // Start with gas deduction and nonce increment
    let initial_gas = tx.gas_limit;
    state.subtract_balance(&tx.from(), tx.value + (initial_gas * tx.gas_price))?;
    state.increment_nonce(&tx.from())?;
    
    // Set up gas tracking
    let mut gas_used = BASE_TX_GAS;
    let mut gas_refund = 0;
    
    // Execute based on transaction type
    let result = match tx.to {
        // Contract creation
        None => {
            gas_used += GAS_CONTRACT_CREATE;
            let code = &tx.data;
            
            // Deploy contract code
            let contract_address = create_contract_address(tx);
            state.create_account(contract_address)?;
            state.set_balance(&contract_address, tx.value)?;
            
            // Execute contract constructor
            let (status, execution_gas, contract_gas_refund) = 
                execute_contract_code(code, &[], &mut context, state)?;
            
            gas_used += execution_gas;
            gas_refund += contract_gas_refund;
            
            if status == ExecutionStatus::Success {
                state.set_code(&contract_address, code)?;
                ExecutionResult::ContractCreated { address: contract_address }
            } else {
                ExecutionResult::Failed
            }
        },
        
        // Regular transfer or contract call
        Some(to) => {
            // Transfer value
            if tx.value > 0 {
                state.add_balance(&to, tx.value)?;
            }
            
            // If recipient is a contract, execute contract code
            if state.is_contract(&to)? {
                let (status, execution_gas, contract_gas_refund) = 
                    execute_contract_code(
                        &state.get_code(&to)?,
                        &tx.data,
                        &mut context, 
                        state
                    )?;
                
                gas_used += execution_gas;
                gas_refund += contract_gas_refund;
                
                if status == ExecutionStatus::Success {
                    ExecutionResult::Success
                } else {
                    ExecutionResult::Failed
                }
            } else {
                // Simple transfer
                ExecutionResult::Success
            }
        }
    };
    
    // Calculate gas refund
    let refunded_gas = std::cmp::min(
        gas_refund, 
        (gas_used - BASE_TX_GAS) / 2
    );
    
    // Calculate unused gas
    let unused_gas = initial_gas - gas_used;
    
    // Refund unused gas and gas refund
    let refund_amount = (unused_gas + refunded_gas) * tx.gas_price;
    if refund_amount > 0 {
        state.add_balance(&tx.from(), refund_amount)?;
    }
    
    // Generate and return receipt
    let receipt = TransactionReceipt {
        transaction_hash: tx.hash(),
        from: tx.from(),
        to: tx.to,
        gas_used,
        gas_limit: initial_gas,
        status: result.status(),
        logs: result.logs().clone(),
        result: result,
    };
    
    Ok(receipt)
}
```

**Design Rationale**:
- **Atomic Execution**: Each transaction either completely succeeds or fails
- **Gas Metering**: Accurately tracks resource usage for fair billing
- **Refund Mechanism**: Returns unused gas to senders
- **Comprehensive Receipts**: Captures all relevant execution outcomes

**For Beginners**: This is like a bank processing a check - they first verify funds, then deduct the amount, perform the requested operation, and finally record the result in your statement.

## 5. Finality and Confirmation

### 5.1 Block Inclusion
Once transactions are executed, the block containing them is proposed to the network.

```rust
fn create_block(
    parent_hash: BlockHash,
    state_root: StateRoot,
    transactions: Vec<SignedTransaction>,
    receipts: Vec<TransactionReceipt>,
    timestamp: Timestamp,
) -> Result<Block> {
    // Calculate transaction root
    let transaction_root = calculate_merkle_root(&transactions);
    
    // Calculate receipt root
    let receipt_root = calculate_merkle_root(&receipts);
    
    // Calculate logs bloom filter
    let logs_bloom = calculate_logs_bloom(&receipts);
    
    // Create block header
    let header = BlockHeader {
        parent_hash,
        state_root,
        transaction_root,
        receipt_root,
        logs_bloom,
        timestamp,
        block_number: get_block_number(parent_hash)? + 1,
        gas_used: receipts.iter().map(|r| r.gas_used).sum(),
        gas_limit: BLOCK_GAS_LIMIT,
        // Other header fields
        // ...existing code...
    };
    
    // Create block
    let block = Block {
        header,
        transactions,
    };
    
    Ok(block)
}
```

**Design Rationale**:
- **Merkle Roots**: Efficient proof structures for transactions and receipts
- **Bloom Filter**: Quick searching for logs without scanning all receipts
- **Complete Metadata**: All necessary information for verification
- **Chain Linkage**: References parent block to maintain blockchain structure

**For Beginners**: This is like compiling all the day's transactions into a ledger page, stamping it with the date and page number, and adding it to the company's accounting book.

### 5.2 Consensus and Finality
The network reaches consensus on the block through the consensus mechanism.

```rust
async fn process_proposed_block(
    block: Block, 
    consensus: &mut ConsensusEngine,
    state: &mut State
) -> Result<BlockStatus> {
    // Verify block structure
    validate_block_structure(&block)?;
    
    // Verify block producer is authorized
    consensus.verify_block_producer(&block)?;
    
    // Execute all transactions to verify state transitions
    let (new_state, receipts) = execute_block_transactions(&block, state)?;
    
    // Verify resulting state root matches
    if new_state.root_hash() != block.header.state_root {
        return Err(Error::StateMismatch);
    }
    
    // Collect attestations from validators
    let attestations = consensus.collect_attestations(&block).await?;
    
    // Determine finality status
    let status = if consensus.is_finalized(&attestations)? {
        BlockStatus::Finalized
    } else {
        BlockStatus::Confirmed
    };
    
    // Update canonical chain
    if status == BlockStatus::Finalized {
        consensus.update_canonical_chain(&block)?;
    }
    
    Ok(status)
}
```

**Design Rationale**:
- **Double Verification**: Re-executes transactions to verify state transition
- **Attestation Collection**: Gathers approvals from network validators
- **Finality Determination**: Uses consensus rules to determine when a block is final
- **Chain Maintenance**: Updates the canonical chain with finalized blocks

**For Beginners**: This is like a group of accountants independently verifying the day's transactions, then signing off that they all agree on the final numbers, making the entries permanent in the company's books.

## 6. Post-Execution Processing

### 6.1 Receipt Storage and Indexing
Transaction receipts are stored for future reference and indexed for efficient queries.

```rust
fn store_transaction_receipts(
    receipts: &[TransactionReceipt], 
    block: &Block,
    db: &mut Database
) -> Result<()> {
    let block_hash = block.hash();
    let block_number = block.header.block_number;
    
    // Store each receipt
    for (index, receipt) in receipts.iter().enumerate() {
        // Store main receipt data
        let key = format!("receipt:{}:{}", block_number, index);
        db.put(key, serialize(receipt)?)?;
        
        // Create index by transaction hash
        let tx_key = format!("tx_receipt:{}", receipt.transaction_hash);
        db.put(tx_key, serialize(&(block_number, index))?)?;
        
        // Index logs by topics
        for log in &receipt.logs {
            for topic in &log.topics {
                let log_key = format!("log:{}:{}", hex::encode(topic), block_number);
                db.put(log_key, serialize(&(block_number, index, log.index))?)?;
            }
        }
        
        // Index by address
        if let Some(address) = receipt.contract_address {
            let addr_key = format!("contract_creation:{}:{}", address, block_number);
            db.put(addr_key, serialize(&(block_number, index))?)?;
        }
        
        let from_key = format!("addr_tx:{}:{}", receipt.from, block_number);
        db.put(from_key, serialize(&(block_number, index))?)?;
        
        if let Some(to) = receipt.to {
            let to_key = format!("addr_tx:{}:{}", to, block_number);
            db.put(to_key, serialize(&(block_number, index))?)?;
        }
    }
    
    Ok(())
}
```

**Design Rationale**:
- **Comprehensive Storage**: Keeps detailed record of all transaction outcomes
- **Multiple Indices**: Enables efficient lookups by hash, address, and topics
- **Bloom Filter Support**: Augments the block-level bloom filter for log filtering
- **Receipt Linking**: Associates receipts with blocks and transactions

**For Beginners**: This is like filing receipts in multiple folders - by date, by vendor, by category - so you can quickly find them no matter what information you have to search with.

### 6.2 Event Notification
External systems are notified of transaction completions and events.

```rust
async fn notify_transaction_completion(
    receipt: &TransactionReceipt,
    notification_system: &NotificationSystem
) -> Result<()> {
    // Create notification payload
    let notification = TransactionNotification {
        transaction_hash: receipt.transaction_hash,
        block_number: receipt.block_number,
        block_hash: receipt.block_hash,
        status: receipt.status,
        gas_used: receipt.gas_used,
        contract_address: receipt.contract_address,
        logs: receipt.logs.clone(),
    };
    
    // Notify subscribers
    notification_system
        .notify_transaction(notification)
        .await?;
    
    // Process event logs for topic-specific notifications
    for log in &receipt.logs {
        notification_system
            .notify_log_event(log)
            .await?;
    }
    
    Ok(())
}
```

**Design Rationale**:
- **Push Notifications**: Proactively informs clients of relevant events
- **Selective Subscription**: Clients only receive events they're interested in
- **Low Latency**: Immediate notification when transactions are processed
- **Decoupling**: Allows external systems to react to blockchain events

**For Beginners**: This is like how you get a text message confirmation when your bank processes a payment - immediate notification that your transaction is complete.

## 7. Transaction Status Lifecycle

### 7.1 Status Tracking
A transaction can exist in multiple states throughout its lifecycle.

```rust
enum TransactionStatus {
    Unknown,              // Transaction not seen by the network
    Pending,              // In mempool, not yet included in block
    Included {            // Included in block but not finalized
        block_hash: BlockHash,
        block_number: BlockNumber,
    },
    Confirmed {           // Included in confirmed block
        block_hash: BlockHash,
        block_number: BlockNumber,
        confirmations: u64,
    },
    Finalized {           // In finalized block, permanently on chain
        block_hash: BlockHash,
        block_number: BlockNumber,
        receipt: TransactionReceipt,
    },
    Failed {              // Transaction reverted or had error
        block_hash: BlockHash,
        block_number: BlockNumber,
        receipt: TransactionReceipt,
        error: String,
    },
    Dropped,              // Removed from mempool without processing
}
```

**Design Rationale**:
- **Clear State Progression**: Well-defined transaction states
- **Sufficient Detail**: Includes all relevant information for each state
- **Error Handling**: Captures failure details for debugging
- **Confirmation Tracking**: Shows progress toward finality

**For Beginners**: This is like tracking a package delivery - your transaction moves through different stages from submission to delivery, with tracking information at each step.

### 7.2 Transaction Lookup
Looking up transaction status and details.

```rust
async fn get_transaction_status(
    tx_hash: &TransactionHash, 
    blockchain: &Blockchain
) -> Result<TransactionStatus> {
    // Check mempool first
    if blockchain.mempool.contains(tx_hash)? {
        return Ok(TransactionStatus::Pending);
    }
    
    // Try to find transaction in blockchain
    if let Some(location) = blockchain.transaction_index.get_transaction_location(tx_hash)? {
        let BlockLocation { hash, number } = location;
        let block_status = blockchain.get_block_status(&hash)?;
        
        match block_status {
            BlockStatus::Pending => {
                Ok(TransactionStatus::Included {
                    block_hash: hash,
                    block_number: number,
                })
            }
            BlockStatus::Confirmed => {
                let current_height = blockchain.get_block_height()?;
                let confirmations = current_height - number;
                
                Ok(TransactionStatus::Confirmed {
                    block_hash: hash,
                    block_number: number,
                    confirmations,
                })
            }
            BlockStatus::Finalized => {
                let receipt = blockchain.get_transaction_receipt(tx_hash)?
                    .ok_or(Error::ReceiptNotFound)?;
                
                if receipt.status == ExecutionStatus::Success {
                    Ok(TransactionStatus::Finalized {
                        block_hash: hash,
                        block_number: number,
                        receipt,
                    })
                } else {
                    let error = format!("Transaction reverted: {:?}", receipt.error);
                    Ok(TransactionStatus::Failed {
                        block_hash: hash,
                        block_number: number,
                        receipt,
                        error,
                    })
                }
            }
        }
    } else {
        // Not in mempool or blockchain
        Ok(TransactionStatus::Unknown)
    }
}
```

**Design Rationale**:
- **Comprehensive Checking**: Looks in mempool and blockchain
- **Current Information**: Calculates confirmations based on current height
- **Detailed Responses**: Provides all relevant information for each status
- **Error Visibility**: Exposes execution errors for debugging

**For Beginners**: This is like checking your bank's website to see if a payment has been processed, pending, or failed.

## 8. References
- **Ethereum Yellow Paper**: Formal specification of blockchain execution
- **EIP-658**: Transaction receipt status field
- **JSONRPC Standard**: For transaction submission and querying
- **Gas Calculation Strategy**: ProzChain gas calculation methodology