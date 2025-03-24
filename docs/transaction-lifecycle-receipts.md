# Transaction Receipts

## Overview

Transaction receipts are the permanent records generated after transaction execution that document the results and side effects of transactions on the ProzChain network. These immutable receipts serve as cryptographic proof of transaction execution, providing crucial information about transaction outcomes, gas usage, event logs, and execution status. By examining transaction receipts, users can verify transaction execution, monitor state changes, and process event data for off-chain systems.

This document explains the structure of transaction receipts, their role in the transaction lifecycle, how they're generated and stored, and the various ways they can be used by developers and applications.

## Receipt Structure

### Core Components

The essential elements of a transaction receipt:

1. **Transaction Hash**:
   - Unique identifier of the executed transaction
   - Cryptographic hash of the transaction data
   - Used for receipt lookups and reference

2. **Transaction Index**:
   - Position of the transaction within its containing block
   - Zero-based index for the first transaction
   - Used for deterministic ordering

3. **Block Information**:
   - Block hash: Hash of the containing block
   - Block number: Height of the containing block
   - Used for locating the transaction in the blockchain

4. **Execution Results**:
   - Status: Success (1) or failure (0)
   - Gas used: Total gas consumed by execution
   - Cumulative gas used: Running total within the block
   - Contract address: For contract creation transactions

5. **Event Logs**:
   - Array of events emitted during execution
   - Indexed topics for efficient filtering
   - Data payload for event parameters
   - Hierarchically structured in a logs bloom filter

### Data Structure Definition

The JSON representation of a transaction receipt:

```json
{
  "transactionHash": "0x5f4f6as87af9875a98f7a5498fa7495a84fa5a98f7a5",
  "transactionIndex": "0x1",
  "blockHash": "0x8f9a7a85f6a87f5a98f7a549a8fa7f5a98f7a5498fa7",
  "blockNumber": "0x429d3b",
  "from": "0xb794f5ea0ba39494ce839613fffba74279579268",
  "to": "0x5a56da779fd725c9c8a6d31271af6137893a77a8",
  "gasUsed": "0x5208",
  "cumulativeGasUsed": "0x71a8",
  "contractAddress": null,
  "logs": [
    {
      "address": "0x5a56da779fd725c9c8a6d31271af6137893a77a8",
      "topics": [
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
        "0x000000000000000000000000b794f5ea0ba39494ce839613fffba74279579268",
        "0x0000000000000000000000008a14c3bc1a351c894c188cbebf33dc911b54583f"
      ],
      "data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
      "blockNumber": "0x429d3b",
      "transactionHash": "0x5f4f6as87af9875a98f7a5498fa7495a84fa5a98f7a5",
      "transactionIndex": "0x1",
      "blockHash": "0x8f9a7a85f6a87f5a98f7a549a8fa7f5a98f7a5498fa7",
      "logIndex": "0x0",
      "removed": false
    }
  ],
  "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
  "status": "0x1",
  "effectiveGasPrice": "0x4a817c800"
}
```

### Type Definitions

The programmatic structure of receipts:

```go
// Basic receipt structure
type Receipt struct {
    // Transaction metadata
    TxHash            common.Hash    `json:"transactionHash"`
    TxIndex           uint           `json:"transactionIndex"`
    BlockHash         common.Hash    `json:"blockHash"`
    BlockNumber       *big.Int       `json:"blockNumber"`
    
    // Transaction parties
    From              common.Address `json:"from"`
    To                *common.Address `json:"to"`
    
    // Gas information
    GasUsed           uint64         `json:"gasUsed"`
    CumulativeGasUsed uint64         `json:"cumulativeGasUsed"`
    EffectiveGasPrice *big.Int       `json:"effectiveGasPrice"`
    
    // Execution results
    ContractAddress   *common.Address `json:"contractAddress"`
    Status            uint64         `json:"status"`
    
    // Event data
    Logs              []*Log         `json:"logs"`
    LogsBloom         Bloom          `json:"logsBloom"`
    
    // For compatibility with legacy systems
    PostState         []byte         `json:"root"`
}

// Log entry structure
type Log struct {
    // Source information
    Address          common.Address  `json:"address"`
    
    // Event data
    Topics           []common.Hash   `json:"topics"`
    Data             []byte          `json:"data"`
    
    // Transaction context
    BlockNumber      uint64          `json:"blockNumber"`
    TxHash           common.Hash     `json:"transactionHash"`
    TxIndex          uint            `json:"transactionIndex"`
    BlockHash        common.Hash     `json:"blockHash"`
    LogIndex         uint            `json:"logIndex"`
    
    // Status 
    Removed          bool            `json:"removed"`
}
```

### ProzChain Receipt Extensions

ProzChain-specific additions to standard receipts:

1. **Zero-Knowledge Proofs**:
   - For confidential transactions
   - Verification success status
   - Commitment references
   - Redacted fields indicator

2. **Layer 2 Metadata**:
   - Cross-layer references
   - Batch inclusion information
   - Rollup/sidechain status
   - Bridge transaction data

3. **Extended Status Information**:
   - Detailed error codes beyond binary status
   - Execution traces references
   - Revert reason strings
   - Execution metrics

4. **Example ProzChain Receipt**:

```json
{
  // Standard fields
  "transactionHash": "0x5f4f6as87af9875a98f7a5498fa7495a84fa5a98f7a5",
  "status": "0x1",
  // ...other standard fields...
  
  // ProzChain extensions
  "extensions": {
    "confidentialTransaction": {
      "proofVerified": true,
      "commitmentIndex": 42,
      "redactedFields": ["value", "to"]
    },
    "detailedStatus": {
      "errorCode": 0,
      "revertReason": null,
      "executionPhase": "complete"
    },
    "layer2Data": {
      "batchNumber": 157,
      "batchPosition": 23,
      "stateRoot": "0x8a7f5a98f7a5498fa7f5a98f7a5498fa7f5a98f7a549"
    }
  }
}
```

## Receipt Generation

### Creation Process

How receipts are constructed during transaction execution:

1. **Initialization**:
   - Created at the start of transaction processing
   - Populated with transaction reference data
   - Block context information added
   - Gas tracking initialized

2. **Runtime Updates**:
   - Gas consumption tracked throughout execution
   - Event logs collected as emitted
   - Bloom filter updated with each log
   - Contract creation address recorded if applicable

3. **Finalization**:
   - Status code set based on execution outcome
   - Final gas calculations performed
   - Logs organized and finalized
   - Receipt cryptographically sealed

4. **Implementation Example**:

```go
// Create transaction receipt after execution
func createReceipt(tx *types.Transaction, blockHash common.Hash, blockNumber, index uint64, gasUsed uint64, status uint64, logs []*types.Log) *types.Receipt {
    // Create bloom filter from logs
    bloom := types.CreateBloom(logs)
    
    // Get transaction sender
    from, _ := types.Sender(signer, tx)
    
    // Prepare contract address (only for contract creations)
    var contractAddress *common.Address
    if tx.To() == nil {
        addr := crypto.CreateAddress(from, tx.Nonce())
        contractAddress = &addr
    }
    
    // Create and return the receipt
    receipt := &types.Receipt{
        Type:              tx.Type(),
        Status:            status,
        CumulativeGasUsed: gasUsed,
        Logs:              logs,
        TxHash:            tx.Hash(),
        GasUsed:           tx.Gas(),
        ContractAddress:   contractAddress,
        BlockHash:         blockHash,
        BlockNumber:       new(big.Int).SetUint64(blockNumber),
        TransactionIndex:  uint(index),
        From:              from,
        To:                tx.To(),
        LogsBloom:         bloom,
        EffectiveGasPrice: tx.EffectiveGasPrice(header.BaseFee),
    }
    
    return receipt
}
```

### Event Log Collection

The process of gathering and organizing event logs:

1. **Event Emission**:
   - Contract code emits events via LOG opcodes
   - Event topics and data stored separately
   - Up to 4 indexed topics per event
   - Arbitrary bytes in data field

2. **Log Storage Process**:
   - Log objects created for each emitted event
   - Transaction context attached to each log
   - Logs ordered by emission sequence
   - Position in block tracked via log index

3. **Bloom Filter Creation**:
   - Probabilistic data structure for efficient filtering
   - Combined from all logs in the receipt
   - Address and all topics contribute to the filter
   - Used for quick log lookup without scanning all receipts

4. **Example Log Generation**:

```go
// Process LOG opcode (simplified)
func (evm *EVM) processLogOpcode(contract *Contract, topics []common.Hash, data []byte) {
    // Create the log object
    log := &types.Log{
        Address:     contract.Address(),
        Topics:      topics,
        Data:        data,
        BlockNumber: evm.Context.BlockNumber.Uint64(),
        TxHash:      evm.TxContext.TxHash,
        TxIndex:     uint(evm.TxContext.TxIndex),
        BlockHash:   evm.Context.BlockHash,
        LogIndex:    uint(evm.StateDB.GetLogSize()),
    }
    
    // Add log to the state
    evm.StateDB.AddLog(log)
}
```

### Status Codes

How transaction success or failure is recorded:

1. **Binary Status Field**:
   - 1 (0x1): Transaction executed successfully
   - 0 (0x0): Transaction failed during execution
   - Pre-Byzantium: Root hash instead of status

2. **Failure Causes**:
   - Out of gas exception
   - Stack underflow/overflow
   - Invalid jump destination
   - Explicit revert calls
   - Other EVM exceptions

3. **Status Determination**:

```go
// Determine transaction status from execution result
func determineStatus(result *ExecutionResult) uint64 {
    // Check for any error
    if result.Err != nil {
        // Special case: don't mark revert with insufficient allowance as failure
        // (ProzChain-specific optimization)
        if isAllowanceRevert(result.Err) {
            // This is a special case where we don't want to mark as failed
            // but include the revert reason in the receipt
            return types.ReceiptStatusSuccessWithRevert
        }
        
        // Regular failure
        return types.ReceiptStatusFailed
    }
    
    // No error means success
    return types.ReceiptStatusSuccessful
}
```

## Receipt Storage and Accessibility

### Receipt Trie

How receipts are organized and stored:

1. **Trie Structure**:
   - Merkle Patricia Trie similar to state trie
   - Indexed by transaction index within block
   - All block receipts combined in single trie
   - Root hash included in block header

2. **Trie Construction**:
   - Sequential addition of receipts as transactions execute
   - RLP encoding of receipt data
   - Path in trie based on transaction index
   - Incremental root calculation

3. **Implementation Approach**:

```go
// Build receipt trie for a block
func BuildReceiptTrie(receipts []*types.Receipt, blockHash common.Hash) (common.Hash, error) {
    // Create new trie
    trie := trie.NewEmpty(triedb)
    
    // Add each receipt to the trie
    for i, receipt := range receipts {
        // Encode receipt index as path
        path := rlp.EncodeToBytes(uint(i))
        
        // Encode full receipt data
        receiptBytes, err := rlp.EncodeToBytes(receipt)
        if err != nil {
            return common.Hash{}, err
        }
        
        // Store in trie
        if err := trie.Update(path, receiptBytes); err != nil {
            return common.Hash{}, err
        }
    }
    
    // Commit trie and return root hash
    return trie.Commit(nil)
}
```

### Receipt Storage Models

Different approaches for receipt data persistence:

1. **Full Storage**:
   - All receipts stored indefinitely
   - Indexed by transaction hash
   - Fast lookup for any historical transaction
   - Highest storage requirements

2. **Pruned Storage**:
   - Recent receipts stored completely
   - Older receipts may be pruned
   - Receipt trie roots maintained for verification
   - Configurable retention policy

3. **Archive vs. Full Nodes**:
   - Archive nodes: Store all receipts indefinitely
   - Full nodes: May prune older receipts
   - Light nodes: Rely on receipt proofs from peers
   - Remote services: API access to receipt data

### Receipt Retrieval

Ways to access transaction receipts:

1. **RPC Methods**:
   - `eth_getTransactionReceipt`: Get receipt by transaction hash
   - `eth_getLogs`: Query for logs matching criteria
   - `eth_getBlockReceipts`: Get all receipts for a block
   - `prozchain_getDetailedReceipt`: Enhanced receipt with additional data

2. **Client Libraries Example**:

```javascript
// Using ethers.js to get a transaction receipt
async function getReceiptWithConfirmation(txHash) {
  // Wait for transaction to be mined and get receipt
  const receipt = await provider.waitForTransaction(txHash);
  
  console.log(`Transaction status: ${receipt.status === 1 ? 'Success' : 'Failed'}`);
  console.log(`Gas used: ${receipt.gasUsed.toString()}`);
  console.log(`Block number: ${receipt.blockNumber}`);
  
  // Check for events/logs
  if (receipt.logs.length > 0) {
    console.log(`Events emitted: ${receipt.logs.length}`);
    
    // Example: Parse ERC-20 Transfer event
    const transferInterface = new ethers.utils.Interface([
      "event Transfer(address indexed from, address indexed to, uint256 value)"
    ]);
    
    for (const log of receipt.logs) {
      try {
        const parsedLog = transferInterface.parseLog(log);
        console.log(`Transfer: ${parsedLog.args.from} → ${parsedLog.args.to}: ${ethers.utils.formatEther(parsedLog.args.value)} tokens`);
      } catch (e) {
        // Not a Transfer event or different format
        continue;
      }
    }
  }
  
  return receipt;
}
```

### Merkle Proofs

Verifying receipts without full block data:

1. **Proof Structure**:
   - Merkle path from receipt to receipt root
   - Block header containing receipt root
   - Receipt data to verify against proof
   - Transaction index in block

2. **Verification Process**:
   - Validate proof against receipt root
   - Confirm block header authenticity
   - Check receipt content hash matches proof
   - Verify transaction index path

3. **Light Client Implementation**:

```go
// Verify a receipt with a merkle proof (simplified)
func VerifyReceiptProof(receiptData []byte, proof [][]byte, receiptRoot common.Hash, txIndex uint) bool {
    // Create path from transaction index
    path := rlp.EncodeToBytes(txIndex)
    
    // Verify the proof leads to receipt root
    key := crypto.Keccak256(path)
    value := crypto.Keccak256(receiptData)
    
    if err := trie.VerifyProof(receiptRoot, key, proof); err != nil {
        return false
    }
    
    // Verify the receipt data matches what we expect
    proofValue := ProcessProof(proof, key)
    return bytes.Equal(proofValue, value)
}
```

## Using Receipts

### Transaction Verification

Confirming transaction execution and outcomes:

1. **Confirmation Status**:
   - Verify transaction included in a block
   - Check execution succeeded (status = 1)
   - Confirm gas used within expectations
   - Validate block confirmations for finality

2. **Revert Detection**:
   - Identify failed transactions (status = 0)
   - Extract revert reason when available
   - Categorize failure types for handling
   - Retry strategies based on failure analysis

3. **Implementation Example**:

```javascript
// Process transaction receipt for confirmation
async function verifyTransactionOutcome(txHash, expectedMinConfirmations) {
  // Get receipt
  const receipt = await provider.getTransactionReceipt(txHash);
  
  // Check if transaction is mined
  if (!receipt) {
    return { status: 'pending', confirmations: 0 };
  }
  
  // Check execution status
  if (receipt.status === 0) {
    // Transaction failed - get reason if possible
    const tx = await provider.getTransaction(txHash);
    let revertReason = 'Unknown reason';
    
    try {
      // Try to extract revert reason
      const result = await provider.call(
        {
          to: tx.to,
          from: tx.from,
          data: tx.data,
          value: tx.value,
          gasPrice: tx.gasPrice,
          gasLimit: tx.gasLimit
        },
        receipt.blockNumber
      );
    } catch (error) {
      revertReason = decodeRevertReason(error);
    }
    
    return { 
      status: 'failed', 
      confirmations: 1, 
      reason: revertReason
    };
  }
  
  // Check confirmations
  const currentBlock = await provider.getBlockNumber();
  const confirmations = currentBlock - receipt.blockNumber + 1;
  
  return {
    status: 'success',
    confirmations,
    finalized: confirmations >= expectedMinConfirmations,
    receipt
  };
}
```

### Event Monitoring

Working with emitted events from transactions:

1. **Log Filtering**:
   - Filter by contract address
   - Filter by event signature (topic0)
   - Filter by indexed parameter values
   - Combine multiple filters

2. **Event Processing**:
   - Parse binary log data into typed values
   - Handle indexed vs. non-indexed parameters
   - Associate events with contract actions
   - Reconstruct off-chain state from events

3. **Event Listening Example**:

```javascript
// Listen for ERC-20 transfer events
function monitorTokenTransfers(tokenAddress) {
  // Create interface with event ABI
  const tokenInterface = new ethers.utils.Interface([
    "event Transfer(address indexed from, address indexed to, uint256 value)"
  ]);
  
  // Create filter for Transfer events from the token
  const filter = {
    address: tokenAddress,
    topics: [
      ethers.utils.id("Transfer(address,address,uint256)"),
      null, // from (any address)
      null  // to (any address)
    ]
  };
  
  // Listen for events
  provider.on(filter, (log) => {
    // Parse the log data
    const parsedLog = tokenInterface.parseLog(log);
    const { from, to, value } = parsedLog.args;
    
    console.log(`Transfer: ${from} → ${to}: ${ethers.utils.formatEther(value)} tokens`);
    
    // Additional processing...
    updateBalances(from, to, value);
    notifyRecipient(to, value);
  });
}
```

### State Change Verification

Using receipts to validate state transitions:

1. **Pre/Post State Analysis**:
   - Simulate transaction before execution
   - Compare expected vs. actual outcomes
   - Verify specific state changes occurred
   - Ensure no unexpected side effects

2. **Balance Change Validation**:
   - Track value transfers
   - Account for gas costs
   - Verify net balance changes
   - Detect unexpected asset movements

3. **Implementation Example**:

```javascript
// Validate expected state changes from a transaction
async function validateStateChanges(txHash, expectedChanges) {
  // Get transaction and receipt
  const [tx, receipt] = await Promise.all([
    provider.getTransaction(txHash),
    provider.getTransactionReceipt(txHash)
  ]);
  
  // Transaction failed
  if (receipt.status === 0) {
    throw new Error('Transaction failed');
  }
  
  // Calculate gas cost
  const gasCost = receipt.gasUsed.mul(tx.gasPrice);
  
  // Check all expected state changes
  for (const change of expectedChanges) {
    switch (change.type) {
      case 'balance':
        // Get balances before and after
        const balanceBefore = await provider.getBalance(
          change.address, 
          receipt.blockNumber - 1
        );
        const balanceAfter = await provider.getBalance(
          change.address, 
          receipt.blockNumber
        );
        
        // Calculate expected change with gas consideration
        let expectedBalance;
        if (change.address.toLowerCase() === tx.from.toLowerCase()) {
          // For sender, account for gas costs
          expectedBalance = balanceBefore
            .sub(gasCost)
            .sub(tx.value || 0)
            .add(change.expectedChange || 0);
        } else {
          expectedBalance = balanceBefore.add(change.expectedChange || 0);
        }
        
        // Verify balance change
        if (!balanceAfter.eq(expectedBalance)) {
          throw new Error(
            `Balance mismatch for ${change.address}: ` +
            `expected ${expectedBalance}, got ${balanceAfter}`
          );
        }
        break;
        
      case 'storage':
        // Get storage value after transaction
        const storageValue = await provider.getStorageAt(
          change.address,
          change.slot,
          receipt.blockNumber
        );
        
        // Verify storage change
        if (storageValue !== change.expectedValue) {
          throw new Error(
            `Storage mismatch for ${change.address} slot ${change.slot}: ` +
            `expected ${change.expectedValue}, got ${storageValue}`
          );
        }
        break;
    }
  }
  
  return true;
}
```

### Application Integration

Integrating receipt processing in applications:

1. **Notification Systems**:
   - Transaction confirmation alerts
   - Event-based triggers
   - Webhook delivery of receipt data
   - Real-time state updates

2. **Off-Chain Indexing**:
   - Log aggregation and storage
   - Event-based state tracking
   - Historical activity recording
   - Analytics and reporting systems

3. **Error Handling**:
   - Transaction failure recovery
   - Automated retry mechanisms
   - User feedback on transaction status
   - Graceful degradation for network issues

## Advanced Receipt Topics

### Receipt Extensions

Enhanced receipt functionality:

1. **Detailed Error Information**:
   - ProzChain-specific error codes
   - Structured error data
   - Call stack information
   - Gas usage breakdown

2. **Receipt Metadata**:
   - Timing information
   - Network conditions
   - Node-specific annotations
   - Off-chain context references

3. **Cross-Chain Receipts**:
   - Multi-chain transaction references
   - Bridge transaction markers
   - Cross-chain finality status
   - Layer 2 settlement confirmation

### Receipt Aggregation

Combining receipts for efficiency:

1. **Batch Transaction Receipts**:
   - Combined receipts for multiple operations
   - Per-operation status tracking
   - Shared gas accounting
   - Atomic execution guarantees

2. **Summarized Receipts**:
   - Condensed receipt format for high-volume systems
   - Essential data preservation
   - Bloom filter optimization
   - Storage-efficient representations

3. **Implementation Approach**:

```go
// Create aggregated receipt for batch transaction
func createBatchReceipt(batchTx *types.BatchTransaction, results []*ExecutionResult, 
                       blockHash common.Hash, blockNumber, index uint64) *types.Receipt {
    // Track total gas used
    totalGasUsed := uint64(0)
    allLogs := make([]*types.Log, 0)
    subReceipts := make([]*types.SubReceipt, len(results))
    
    // Overall status starts as success
    overallStatus := types.ReceiptStatusSuccessful
    
    // Process each operation in the batch
    for i, result := range results {
        // Add this operation's gas usage
        totalGasUsed += result.GasUsed
        
        // Determine status
        status := types.ReceiptStatusSuccessful
        if result.Err != nil {
            status = types.ReceiptStatusFailed
            overallStatus = types.ReceiptStatusFailed // Any failure means batch failed
        }
        
        // Create sub-receipt for this operation
        subReceipts[i] = &types.SubReceipt{
            Index:     uint64(i),
            Status:    status,
            GasUsed:   result.GasUsed,
            Logs:      result.Logs,
            ReturnData: result.ReturnData,
        }
        
        // Add logs to the combined list
        allLogs = append(allLogs, result.Logs...)
    }
    
    // Create bloom filter from all logs
    bloom := types.CreateBloom(allLogs)
    
    // Create main receipt
    receipt := &types.Receipt{
        Type:              types.BatchTxType,
        Status:            overallStatus,
        CumulativeGasUsed: totalGasUsed,
        Logs:              allLogs,
        TxHash:            batchTx.Hash(),
        GasUsed:           totalGasUsed,
        BlockHash:         blockHash,
        BlockNumber:       new(big.Int).SetUint64(blockNumber),
        TransactionIndex:  uint(index),
        LogsBloom:         bloom,
        
        // Batch-specific extension
        SubReceipts:       subReceipts,
    }
    
    return receipt
}
```

### Historical Compatibility

Handling receipt format changes:

1. **Legacy Receipt Formats**:
   - Pre-Byzantium receipts (root hash instead of status)
   - Missing fields in older formats
   - Receipt type versioning
   - Automatic format conversion

2. **EIP-658 Transition**:
   - Change from receipt root to status code
   - Backward compatibility handling
   - Status inference from root for old transactions
   - Migration strategies for applications

3. **Client Compatibility**:
   - Handling format differences between clients
   - API format standardization
   - Format negotiation mechanisms
   - Receipt translation services

## Security and Compliance

### Receipt Integrity

Ensuring receipt data validity:

1. **Cryptographic Verification**:
   - Receipt trie validation against block header
   - Hash consistency checking
   - Digital signature verification
   - Tamper detection mechanisms

2. **Receipt Chain Validation**:
   - Block header sequence verification
   - Receipt position within block validation
   - Cross-reference with transaction data
   - Confirmation depth requirements

3. **Implementation Example**:

```go
// Verify receipt integrity (simplified)
func VerifyReceipt(receipt *types.Receipt, block *types.Block) error {
    // Find transaction in block
    var tx *types.Transaction
    for i, transaction := range block.Transactions() {
        if transaction.Hash() == receipt.TxHash {
            tx = transaction
            
            // Verify transaction index
            if uint(i) != receipt.TransactionIndex {
                return errors.New("transaction index mismatch")
            }
            break
        }
    }
    
    if tx == nil {
        return errors.New("transaction not found in specified block")
    }
    
    // Verify block references
    if receipt.BlockHash != block.Hash() {
        return errors.New("block hash mismatch")
    }
    
    if receipt.BlockNumber.Uint64() != block.NumberU64() {
        return errors.New("block number mismatch")
    }
    
    // Verify logs bloom
    calculatedBloom := types.CreateBloom(receipt.Logs)
    if calculatedBloom != receipt.LogsBloom {
        return errors.New("logs bloom mismatch")
    }
    
    // Additional checks could include:
    // - Verify receipt is in the receipt trie of block
    // - Validate contract address calculation
    // - Check gas used is reasonable
    
    return nil
}
```

### Compliance and Auditing

Using receipts for regulatory and business purposes:

1. **Audit Trail Creation**:
   - Transaction history recording
   - Immutable evidence of operations
   - Chronological operation logs
   - Business event tracking

2. **Compliance Reporting**:
   - Transaction categorization
   - Regulatory reporting extraction
   - Evidence of rule adherence
   - Automated compliance checks

3. **Data Retention Policies**:
   - Receipt archival requirements
   - Long-term storage solutions
   - Retrieval and access controls
   - Receipt data lifecycle management

### Receipt Privacy

Handling sensitive transaction information:

1. **Confidential Transaction Receipts**:
   - Selective disclosure of receipt data
   - Zero-knowledge proof validation
   - Encrypted event parameters
   - Privacy-preserving audit capabilities

2. **Private Metadata**:
   - Separating public and private receipt components
   - Permission-based access to sensitive data
   - Off-chain confidential information storage
   - Key-based selective disclosure

3. **Regulatory Compliance**:
   - Authorized accessor mechanisms
   - Legally compliant disclosure systems
   - Privacy-preserving audit tools
   - Regulated data handling procedures

## Receipt Tooling and Best Practices

### Receipt Indexing

Optimizing receipt discovery and analysis:

1. **Indexing Strategies**:
   - Transaction hash indexing
   - Address-based indexing (from/to)
   - Event signature indexing
   - Compound indices for complex queries

2. **Database Schemas**:
   - Relational storage optimization
   - NoSQL document storage approaches
   - Time-series organization
   - Graph relationships for contract interactions

3. **Query Optimization**:
   - Bloom filter pre-filtering
   - Caching strategies
   - Materialized views for common queries
   - Pagination and result limiting

### Error Handling

Managing transaction failures with receipts:

1. **Revert Reason Extraction**:
   - Standard error format decoding
   - Custom error parsing
   - ABI-encoded error interpretation
   - Human-readable error reporting

2. **Failure Classification**:
   - Gas-related failures
   - Business logic errors
   - Access control failures
   - Technical execution errors

3. **Implementation Example**:

```javascript
// Extract and decode revert reasons from failed transactions
async function getRevertReason(txHash) {
  const tx = await provider.getTransaction(txHash);
  const receipt = await provider.getTransactionReceipt(txHash);
  
  // Check if transaction failed
  if (!receipt || receipt.status !== 0) {
    return { failed: false };
  }
  
  try {
    // Try to replay the transaction to get the error
    await provider.call(
      {
        from: tx.from,
        to: tx.to,
        data: tx.data,
        value: tx.value,
        gasLimit: tx.gasLimit,
        gasPrice: tx.gasPrice
      },
      receipt.blockNumber
    );
    
    // If we reach here, no error was thrown
    return { failed: true, reason: "Unknown error - call succeeded on replay" };
  } catch (error) {
    // Parse the error
    const errorData = error.data || error.error?.data;
    
    if (!errorData) {
      return { failed: true, reason: error.message };
    }
    
    // Try to decode the revert reason
    const reasonHex = errorData.substring(errorData.indexOf('0x'));
    
    // Standard revert reason format: Error(string)
    if (reasonHex.length >= 138) {
      // Extract the string length and data
      const strLen = parseInt(reasonHex.slice(2 + 8, 2 + 8 + 64), 16);
      if (strLen > 0 && 2 + 8 + 64 + (strLen * 2) <= reasonHex.length) {
        const reasonText = Buffer.from(
          reasonHex.slice(2 + 8 + 64, 2 + 8 + 64 + (strLen * 2)),
          'hex'
        ).toString('utf8');
        
        return { failed: true, reason: reasonText, raw: reasonHex };
      }
    }
    
    // Try to match known error signatures
    // Customize with your contract's custom errors
    const errorSigs = {
      '0x08c379a0': 'Error(string)',
      '0x4e487b71': 'Panic(uint256)',
      // Add your contract's custom error signatures
    };
    
    const errorSig = reasonHex.substring(0, 10);
    const knownError = errorSigs[errorSig];
    
    if (knownError) {
      return { 
        failed: true, 
        reason: `${knownError} - ${reasonHex.slice(10)}`,
        raw: reasonHex
      };
    }
    
    // Return raw data if we can't decode
    return { failed: true, reason: `Unknown error format: ${reasonHex}` };
  }
}
```

### Receipt UX Integration

Making receipts user-friendly in applications:

1. **Status Visualization**:
   - Transaction success/failure indicators
   - Confirmation progress visualization
   - Gas usage reporting
   - Execution outcome summaries

2. **Event Interpretation**:
   - Human-readable event formatting
   - Event aggregation and summarization
   - Domain-specific event presentation
   - Timeline-based event visualization

3. **Receipt Links**:
   - Block explorer deep links
   - Transaction history organization
   - Related transaction grouping
   - Action-consequence demonstrations

4. **Implementation Example**:

```javascript
// Create user-friendly receipt summary for frontend display
function createReceiptSummary(receipt, tx, contractInterfaces) {
  // Basic transaction info
  const summary = {
    status: receipt.status === 1 ? 'Success' : 'Failed',
    hash: receipt.transactionHash,
    confirmations: currentBlockNumber - receipt.blockNumber,
    from: tx.from,
    to: tx.to || 'Contract Creation',
    value: tx.value ? ethers.utils.formatEther(tx.value) + ' ETH' : '0 ETH',
    gasCost: ethers.utils.formatEther(
      receipt.effectiveGasPrice.mul(receipt.gasUsed)
    ) + ' ETH',
    blockExplorerLink: `https://explorer.prozchain.net/tx/${receipt.transactionHash}`,
    events: []
  };
  
  // Add contract creation info if applicable
  if (receipt.contractAddress) {
    summary.contractCreated = receipt.contractAddress;
    summary.contractLink = `https://explorer.prozchain.net/address/${receipt.contractAddress}`;
  }
  
  // Process logs to readable events
  if (receipt.logs && receipt.logs.length > 0) {
    // Group logs by contract address
    const logsByContract = {};
    for (const log of receipt.logs) {
      if (!logsByContract[log.address]) {
        logsByContract[log.address] = [];
      }
      logsByContract[log.address].push(log);
    }
    
    // Process logs for each contract
    for (const [address, logs] of Object.entries(logsByContract)) {
      // Find matching interface for this contract
      const iface = contractInterfaces[address] || 
                   findMatchingInterface(logs[0], contractInterfaces);
      
      for (const log of logs) {
        try {
          // Try to parse with known interface
          if (iface) {
            const parsedLog = iface.parseLog(log);
            
            // Create friendly event representation
            const event = {
              name: parsedLog.name,
              contractName: iface.contractName || 'Unknown Contract',
              contractAddress: log.address,
              args: {}
            };
            
            // Format arguments in a human-readable way
            for (const [key, value] of Object.entries(parsedLog.args)) {
              if (!isNaN(parseInt(key))) continue; // Skip numeric indexes
              
              // Format based on type
              if (ethers.BigNumber.isBigNumber(value)) {
                // Try to detect if this is a token amount
                if (parsedLog.name.includes('Transfer') && 
                    (key.includes('amount') || key.includes('value'))) {
                  event.args[key] = formatTokenAmount(value, iface.decimals || 18);
                } else {
                  event.args[key] = value.toString();
                }
              } else if (typeof value === 'string' && value.startsWith('0x') 
                        && value.length === 42) {
                // Likely an address
                event.args[key] = shortenAddress(value);
              } else {
                event.args[key] = value.toString();
              }
            }
            
            summary.events.push(event);
          } else {
            // Unknown event format
            summary.events.push({
              name: 'Unknown Event',
              contractAddress: log.address,
              topics: log.topics,
              data: log.data
            });
          }
        } catch (e) {
          // Failed to parse this log
          summary.events.push({
            name: 'Unparseable Event',
            contractAddress: log.address,
            error: e.message
          });
        }
      }
    }
  }
  
  return summary;
}
```

### Receipt Analysis

Extracting insights from transaction receipts:

1. **Gas Usage Analysis**:
   - Operation cost breakdown
   - Efficiency comparison
   - Optimization opportunities
   - Cost trend monitoring

2. **Behavior Pattern Detection**:
   - Contract interaction patterns
   - User activity profiling
   - Anomaly detection
   - Usage pattern evolution

3. **Performance Metrics**:
   - Execution time estimation
   - Resource consumption tracking
   - Scaling bottleneck identification
   - Comparative benchmark analysis

## Conclusion

Transaction receipts form the permanent record of transaction execution in ProzChain, providing essential information about execution outcomes, emitted events, and gas consumption. They play a crucial role in transaction verification, event monitoring, and state change validation, serving as the foundation for many blockchain application features.

The receipt system balances multiple requirements: cryptographic verifiability for security, efficient storage and retrieval for performance, and rich event data for application usability. Through careful design and optimization, ProzChain's receipt infrastructure enables a wide range of use cases from simple transaction confirmation to complex event-driven applications.

Understanding how to work with transaction receipts effectively is essential for blockchain developers, allowing them to build reliable, user-friendly applications that can respond appropriately to transaction outcomes and leverage the valuable data contained in event logs.

In the next document, [Exceptional Conditions](./transaction-lifecycle-exceptions.md), we'll explore how ProzChain handles various error conditions and edge cases that can occur during transaction processing.
