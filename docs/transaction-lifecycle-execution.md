# Transaction Execution

## Overview

Transaction execution is the process by which the operations specified in a transaction are performed, causing state transitions in the ProzChain blockchain. This phase occurs after a transaction has been included in a block and represents the actual computation and state modifications that implement the transaction's intent.

This document explores the execution environment, how different transaction types are processed, the computational model that governs execution, gas mechanics, and the outcomes of successful and failed executions. Understanding transaction execution is crucial for developers creating smart contracts and applications on ProzChain.

## Execution Environment

### Execution Context

The environment in which transactions are executed:

1. **Block Context**:
   - Block number
   - Timestamp
   - Difficulty
   - Gas limit
   - Block header information

2. **Transaction Context**:
   - Sender (recovered from signature)
   - Recipient (target address)
   - Value
   - Data payload
   - Gas limit and price
   - Nonce

3. **State Context**:
   - World state at beginning of transaction
   - Account balances
   - Contract code
   - Storage values
   - Nonce values

### The ProzChain Virtual Machine (PVM)

The execution engine for transactions:

1. **Architecture**:
   - Stack-based virtual machine
   - 256-bit word size
   - Deterministic execution
   - Metered computation (gas)
   - Isolated execution environment

2. **Implementation**:
   - Interpreter-based execution model
   - Just-in-time compilation for optimization
   - Opcode-by-opcode processing
   - Stack, memory, and storage access

3. **Runtime Structure**:

```go
// Simplified structure of the EVM
type EVM struct {
    // Execution context
    Context BlockContext
    TxContext TxContext
    
    // State access
    StateDB StateDB
    
    // Depth tracks call depth
    depth int
    
    // Gas metering
    interpreter *Interpreter
    
    // Configuration
    Config Config
    
    // Precompiled contracts
    precompiles map[common.Address]PrecompiledContract
}

// BlockContext contains information about the block being processed
type BlockContext struct {
    BlockNumber *big.Int
    Time *big.Int
    Difficulty *big.Int
    GasLimit uint64
    Coinbase common.Address
}

// TxContext contains information about the transaction being executed
type TxContext struct {
    Origin common.Address
    GasPrice *big.Int
    TxHash common.Hash
}
```

### Precompiled Contracts

Built-in functions providing optimized operations:

1. **Standard Precompiles**:
   - ECRECOVER: Public key recovery from signature
   - SHA256: SHA-256 hash function
   - RIPEMD160: RIPEMD-160 hash function
   - IDENTITY: Data copy function
   - MODEXP: Modular exponentiation
   - ECADD/ECMUL: Elliptic curve operations
   - PAIRING: Elliptic curve pairing checks
   - BLAKE2F: BLAKE2 compression function

2. **ProzChain Extended Precompiles**:
   - ZK-VERIFY: Zero-knowledge proof verification
   - BLS-AGGREGATE: BLS signature aggregation
   - STATE-PROOF: State proof verification
   - CONFIDENTIAL-TRANSFER: Privacy-preserving transfers

3. **Execution Cost Model**:
   - Fixed costs for simple operations
   - Input-dependent costs for complex computations
   - Gas refunds for specific operations
   - Optimization for frequently used cryptographic functions

## Transaction Types and Execution

### Simple Value Transfers

Execution of basic ETH transfers:

1. **Execution Steps**:
   - Validate sender has sufficient balance
   - Verify transaction nonce matches sender's current nonce
   - Deduct value from sender
   - Add value to recipient
   - Pay transaction fee
   - Increment sender's nonce

2. **Implementation Example**:

```go
// Process a simple value transfer
func executeValueTransfer(evm *EVM, msg Message) (*ExecutionResult, error) {
    // Get sender and recipient
    sender := msg.From()
    recipient := *msg.To()
    
    // Check sender balance
    if evm.StateDB.GetBalance(sender).Cmp(msg.Value()) < 0 {
        return nil, ErrInsufficientBalance
    }
    
    // Check nonce
    if evm.StateDB.GetNonce(sender) != msg.Nonce() {
        return nil, ErrNonceMismatch
    }
    
    // Execute transfer
    evm.StateDB.SubBalance(sender, msg.Value())
    evm.StateDB.AddBalance(recipient, msg.Value())
    
    // Update nonce
    evm.StateDB.SetNonce(sender, evm.StateDB.GetNonce(sender)+1)
    
    return &ExecutionResult{
        UsedGas:    params.TxGas,
        Err:        nil,
    }, nil
}
```

3. **Edge Cases**:
   - Transfers to non-existent accounts create new accounts
   - Zero-value transfers are valid and update nonce
   - Transfers to contract addresses trigger contract code execution
   - Self-transfers are processed normally (deduct and add to same address)

### Contract Deployment

Execution of contract creation transactions:

1. **Execution Sequence**:
   - Validate transaction (nonce, gas, etc.)
   - Generate contract address
   - Execute contract initialization code
   - Store resulting contract code
   - Set initial storage values
   - Emit contract creation event

2. **Code Initialization**:
   - Constructor parameters passed in transaction data
   - Initialization code returns runtime bytecode
   - Storage slots initialized during construction
   - Constructor code is not stored, only runtime code

3. **Implementation Example**:

```go
// Create a new contract
func (evm *EVM) Create(caller ContractRef, code []byte, value *big.Int, gas uint64) (ret []byte, contractAddr common.Address, leftOverGas uint64, err error) {
    // Create address for the new contract
    nonce := evm.StateDB.GetNonce(caller.Address())
    contractAddr = crypto.CreateAddress(caller.Address(), nonce)
    
    // Initialize new contract
    contract := NewContract(caller, AccountRef(contractAddr), value, gas)
    contract.SetCallCode(&contractAddr, crypto.Keccak256Hash(code), code)
    
    // Ensure there's no existing code
    if evm.StateDB.GetCodeSize(contractAddr) != 0 {
        return nil, common.Address{}, 0, ErrContractAddressCollision
    }
    
    // Increment nonce
    evm.StateDB.SetNonce(caller.Address(), nonce+1)
    
    // Transfer value to contract
    evm.StateDB.CreateAccount(contractAddr)
    if value.Sign() != 0 {
        evm.StateDB.TransferBalance(caller.Address(), contractAddr, value)
    }
    
    // Execute init code and get runtime code
    ret, err = evm.interpreter.Run(contract, nil, false)
    
    // Handle execution result
    if err != nil {
        return nil, common.Address{}, contract.Gas, err
    }
    
    // Store contract code
    createDataGas := uint64(len(ret)) * params.CreateDataGas
    if contract.UseGas(createDataGas) {
        evm.StateDB.SetCode(contractAddr, ret)
    } else {
        return nil, common.Address{}, 0, ErrCodeStoreOutOfGas
    }
    
    return ret, contractAddr, contract.Gas, nil
}
```

4. **Contract Creation Limits**:
   - Maximum code size: 24KB (EIP-170)
   - Gas costs proportional to code size
   - Stack depth limit applies
   - Special gas cost for contract creation

### Contract Method Calls

Execution of transactions calling contract functions:

1. **Execution Steps**:
   - Decode function selector (first 4 bytes of data)
   - Identify target function in contract code
   - Parse and validate input parameters
   - Execute contract code
   - Process state changes
   - Capture return values

2. **Function Resolution**:
   - Function selector: keccak256(function signature) & 0xFFFFFFFF
   - ABI-encoded parameters following selector
   - Dynamic types handling
   - Error handling for invalid selectors

3. **Implementation Example**:

```go
// Call a contract method
func (evm *EVM) Call(caller ContractRef, addr common.Address, input []byte, gas uint64, value *big.Int) (ret []byte, leftOverGas uint64, err error) {
    // Check if account exists
    if !evm.StateDB.Exist(addr) {
        return nil, gas, nil
    }
    
    // Transfer value if non-zero
    if value.Sign() != 0 {
        // Perform transfer
        evm.StateDB.SubBalance(caller.Address(), value)
        evm.StateDB.AddBalance(addr, value)
    }
    
    // Get contract code
    code := evm.StateDB.GetCode(addr)
    if len(code) == 0 {
        return nil, gas, nil // Nothing to execute
    }
    
    // Create contract for execution
    contract := NewContract(caller, AccountRef(addr), value, gas)
    contract.SetCallCode(&addr, crypto.Keccak256Hash(code), code)
    
    // Execute contract code
    ret, err = evm.interpreter.Run(contract, input, false)
    
    return ret, contract.Gas, err
}
```

4. **External Contract Interactions**:
   - CALL, STATICCALL, DELEGATECALL, and CALLCODE opcodes
   - Message passing between contracts
   - Value transfers in contract calls
   - Reentrancy considerations

### Special Transaction Types

ProzChain's advanced transaction formats:

1. **Batch Transactions**:
   - Multiple operations in single transaction
   - Shared nonce and signature
   - Atomic execution (all or nothing)
   - Gas sharing across operations

2. **Confidential Transactions**:
   - Zero-knowledge proof verification
   - State encryption and decryption
   - Selective disclosure of transaction details
   - Special validation rules

3. **Implementation Example**:

```go
// Process a batch transaction
func executeBatchTransaction(evm *EVM, tx *types.BatchTransaction) ([]*ExecutionResult, error) {
    // Initialize results array
    results := make([]*ExecutionResult, len(tx.Operations))
    
    // Create snapshot for atomic execution
    snapshot := evm.StateDB.Snapshot()
    
    // Track total gas used
    totalGasUsed := uint64(0)
    
    // Execute each operation
    for i, op := range tx.Operations {
        // Create message from operation
        msg := NewMessage(
            tx.From(),
            op.To,
            tx.Nonce(), // Same nonce for all operations
            op.Value,
            tx.GasLimit - totalGasUsed,
            tx.GasPrice,
            op.Data,
            false,
        )
        
        // Execute operation
        result, err := ApplyMessage(evm, msg)
        results[i] = result
        
        // Track gas
        totalGasUsed += result.UsedGas
        
        // If any operation fails, revert all
        if err != nil {
            // Revert all state changes
            evm.StateDB.RevertToSnapshot(snapshot)
            
            // Return error and partial results
            return results, err
        }
    }
    
    // All operations succeeded, return results
    return results, nil
}
```

## Virtual Machine Operations

### Opcode Execution

How instructions are processed in the PVM:

1. **Opcode Processing Cycle**:
   - Fetch opcode from bytecode
   - Decode instruction
   - Execute operation
   - Update state (stack, memory, storage)
   - Measure and charge gas
   - Increment program counter

2. **Supported Operations**:
   - Arithmetic: ADD, SUB, MUL, DIV, etc.
   - Logic: AND, OR, XOR, NOT
   - Comparison: LT, GT, EQ, etc.
   - Environment: BLOCKHASH, TIMESTAMP, NUMBER, etc.
   - Memory: MLOAD, MSTORE, MSTORE8
   - Storage: SLOAD, SSTORE
   - Control flow: JUMP, JUMPI, PC, STOP
   - Stack: PUSH, POP, DUP, SWAP
   - Contract interaction: CALL, STATICCALL, CREATE, etc.

3. **Implementation Example**:

```go
// Simplified opcode execution loop
func (in *Interpreter) Run(contract *Contract, input []byte, readOnly bool) (ret []byte, err error) {
    // Initialize execution context
    pc := uint64(0)
    stack := newstack()
    memory := newMemory()
    
    // Initialize return data
    returnData := make([]byte, 0)
    
    // Get contract code
    code := contract.Code
    
    // Main execution loop
    for {
        // Check if we've reached the end of code
        if pc >= uint64(len(code)) {
            break
        }
        
        // Get current opcode
        op := code[pc]
        operation := in.table[op]
        
        // Check if operation is valid
        if !operation.valid {
            return nil, ErrInvalidOpcode
        }
        
        // Check if we have enough gas
        if !contract.UseGas(operation.gas) {
            return nil, ErrOutOfGas
        }
        
        // Execute the operation
        res, err := operation.execute(&pc, in, contract, memory, stack)
        
        // Handle execution result
        if err != nil {
            return nil, err
        }
        
        // Update return data if applicable
        if len(res) > 0 {
            returnData = res
        }
        
        // Advance program counter for most operations
        if operation.halts {
            break
        }
    }
    
    return returnData, nil
}
```

### Memory Management

How the PVM manages execution memory:

1. **Memory Model**:
   - Byte-addressable linear memory
   - Dynamically expandable
   - Initially empty (all zeros)
   - Word-aligned operations (32 bytes)
   - No persistence between calls

2. **Memory Operations**:
   - MLOAD: Load 32 bytes from memory
   - MSTORE: Store 32 bytes to memory
   - MSTORE8: Store 1 byte to memory
   - Memory expansion costs

3. **Memory Expansion Costs**:

```go
// Calculate memory expansion cost
func memoryGasCost(mem *Memory, newSize uint64) uint64 {
    if newSize == 0 {
        return 0
    }
    
    // Check if expansion needed
    if uint64(mem.Len()) < newSize {
        // Memory expansion formula from Yellow Paper
        oldSize := toWordSize(uint64(mem.Len()))
        newSize = toWordSize(newSize)
        
        // Calculate cost
        // cost = (newSize^2)/512 - (oldSize^2)/512 + 3*newSize
        oldSizeSquared := oldSize * oldSize
        newSizeSquared := newSize * newSize
        
        cost := (newSizeSquared - oldSizeSquared) / 512
        cost += 3 * (newSize - oldSize)
        
        return cost
    }
    
    return 0
}
```

### Stack Operations

Stack manipulation in the PVM:

1. **Stack Properties**:
   - 1024 element maximum depth
   - 256-bit words (32 bytes)
   - Last-in, first-out structure
   - Zero-initialized elements

2. **Stack Instructions**:
   - PUSHx: Place x bytes on stack (PUSH1-PUSH32)
   - POP: Remove top item from stack
   - DUPx: Duplicate xth stack item (DUP1-DUP16)
   - SWAPx: Swap top with xth stack item (SWAP1-SWAP16)

3. **Stack Error Conditions**:
   - Stack underflow (pop from empty stack)
   - Stack overflow (push beyond max depth)
   - Invalid access (DUP or SWAP beyond stack depth)

### Storage Operations

How contracts interact with persistent storage:

1. **Storage Structure**:
   - Key-value store (256-bit keys to 256-bit values)
   - Contract-specific storage (isolated by address)
   - Initially zero for all keys
   - Persistent between calls

2. **Storage Instructions**:
   - SLOAD: Load value from storage
   - SSTORE: Store value to storage

3. **Storage Gas Costs**:
   - SLOAD: 800 gas (cold access), 100 gas (warm access)
   - SSTORE: Complex cost model based on current value
   - Storage refunds for clearing (setting to zero)
   - EIP-2200 net metering for same-slot updates

4. **Implementation Example**:

```go
// Calculate gas cost for SSTORE operation
func calculateSStoreGasCost(evm *EVM, contract *Contract, key, value, current common.Hash) (uint64, error) {
    // Get original value at the beginning of transaction
    original := evm.StateDB.GetOriginalStateValue(contract.Address(), key)
    
    // Check if slot is warm (accessed before in this transaction)
    isWarm := evm.StateDB.IsSlotWarmed(contract.Address(), key)
    
    var cost uint64
    
    // EIP-2200 logic
    if current == value { // No-op case
        cost = params.WarmStorageReadCost // 100 gas
    } else if current == original { // Reset to original value
        if original == (common.Hash{}) { // Setting slot that was empty
            cost = params.SstoreSetGas // 20000 gas
        } else if value == (common.Hash{}) { // Clearing slot
            evm.StateDB.AddRefund(params.SstoreClearRefund) // Refund
            cost = params.SstoreResetGas // 5000 gas
        } else { // Changing to different non-zero value
            cost = params.SstoreResetGas // 5000 gas
        }
    } else { // Changing already modified value
        if original != (common.Hash{}) {
            if current == (common.Hash{}) { // Recreating a cleared slot
                evm.StateDB.SubRefund(params.SstoreClearRefund)
            } else if value == (common.Hash{}) { // Clearing slot
                evm.StateDB.AddRefund(params.SstoreClearRefund)
            }
        }
        cost = params.WarmStorageReadCost // 100 gas
    }
    
    // Add cold access cost if this is first access to slot
    if !isWarm {
        cost += params.ColdSloadCost - params.WarmStorageReadCost
        evm.StateDB.MarkSlotWarmed(contract.Address(), key)
    }
    
    return cost, nil
}
```

## Gas Mechanics

### Gas Calculation

How gas costs are determined:

1. **Base Gas Costs**:
   - Intrinsic gas: 21,000 for regular transactions
   - Contract creation: 32,000 + intrinsic gas
   - Data bytes: 4 gas per zero byte, 16 gas per non-zero byte
   - Access list: 2,400 per address + 1,900 per storage key

2. **Operation Gas**:
   - Fixed costs for simple operations
   - Dynamic costs for complex operations
   - Memory expansion costs
   - Storage interaction costs

3. **Implementation Example**:

```go
// Calculate intrinsic gas for a transaction
func IntrinsicGas(data []byte, accessList types.AccessList, isContractCreation bool) (uint64, error) {
    // Start with base cost
    gas := params.TxGas // 21,000 for regular tx
    
    // Add contract creation cost if applicable
    if isContractCreation {
        gas += params.TxCreationGas // 32,000 additional
    }
    
    // Add cost for transaction data
    if len(data) > 0 {
        // Count zero and non-zero bytes
        var nz uint64 = 0
        for _, byt := range data {
            if byt != 0 {
                nz++
            }
        }
        
        // Calculate data costs
        nonZeroGas := nz * params.TxDataNonZeroGas
        zeroGas := (uint64(len(data)) - nz) * params.TxDataZeroGas
        
        gas += nonZeroGas + zeroGas
    }
    
    // Add access list costs if present
    if accessList != nil {
        gas += uint64(len(accessList)) * params.AccessListAddressGas
        
        storageKeysCount := 0
        for _, access := range accessList {
            storageKeysCount += len(access.StorageKeys)
        }
        
        gas += uint64(storageKeysCount) * params.AccessListStorageKeyGas
    }
    
    return gas, nil
}
```

### Gas Consumption Tracking

Monitoring and limiting gas usage:

1. **Gas Accounting**:
   - Initial gas allocation from transaction gas limit
   - Deduction per operation executed
   - Refunds for specific operations
   - Out of gas exception when depleted

2. **Gas Refund Mechanism**:
   - Storage slot clearing (SSTORE to zero)
   - Contract self-destruction (SELFDESTRUCT)
   - Maximum refund of 50% of used gas
   - Applied at end of execution

3. **Implementation Example**:

```go
// Contract gas tracking and consumption
type Contract struct {
    Gas uint64
    // Other fields omitted for brevity
}

// UseGas attempts to consume the specified amount of gas
func (c *Contract) UseGas(amount uint64) bool {
    if c.Gas < amount {
        c.Gas = 0
        return false // Out of gas
    }
    c.Gas -= amount
    return true
}

// Apply gas refund at end of execution
func applyRefund(gasUsed, gasLimit uint64, refund uint64) uint64 {
    // Maximum refund is half of gas used
    maxRefund := gasUsed / 2
    
    // Apply actual refund (capped at max)
    if refund > maxRefund {
        refund = maxRefund
    }
    
    // Return unused gas + refund
    remainingGas := gasLimit - gasUsed
    return remainingGas + refund
}
```

### Gas Fee Calculation

How transaction fees are computed:

1. **Legacy Gas Fee Model**:
   - `gas_used * gas_price`
   - Fixed gas price throughout execution
   - Sender pays full amount regardless of actual usage
   - Validator receives entire fee

2. **EIP-1559 Fee Model**:
   - Base fee: Algorithmically determined, burned
   - Priority fee: Miner tip, specified by sender
   - `gas_used * (base_fee + priority_fee)`
   - Sender pays full amount regardless of actual usage
   - Validator receives only priority fee portion

3. **ProzChain Fee Distribution**:
   - 70% of base fee burned
   - 20% of base fee to treasury
   - 10% of base fee to validators
   - 100% of priority fee to validators
   - Special handling for zkEVM privacy circuits

## Execution Outcomes

### Successful Execution

Results of properly completed transactions:

1. **State Changes**:
   - Account balance updates
   - Storage modifications
   - Code deployment
   - Nonce incrementation

2. **Return Values**:
   - Function return data
   - Contract creation address
   - Event logs emission
   - Gas usage and refunds

3. **Receipt Generation**:
   - Transaction hash and block information
   - Execution status (success)
   - Gas used and effective gas price
   - Logs and bloom filter
   - Contract address (for creation transactions)

### Failed Execution

Handling execution failures:

1. **Failure Types**:
   - Out of gas exception
   - Reverted transactions
   - Invalid operations
   - Stack errors
   - Invalid jump destinations

2. **State Reversion**:
   - All state changes rolled back
   - Return to pre-execution state
   - Nonce still incremented
   - Gas still consumed (up to gas limit)

3. **Error Reporting**:
   - Error code in receipt
   - Revert reason (if provided)
   - Gas used until failure
   - Stack trace (for debugging)

4. **Implementation Example**:

```go
// Handle execution failure
func handleExecutionFailure(err error, tx *types.Transaction, state *state.StateDB, gasUsed uint64) *types.Receipt {
    // Create receipt showing failure
    receipt := &types.Receipt{
        Type:              tx.Type(),
        Status:            types.ReceiptStatusFailed,
        CumulativeGasUsed: gasUsed,
        Logs:              []*types.Log{},
        TxHash:            tx.Hash(),
        GasUsed:           gasUsed,
        // Other fields...
    }
    
    // Extract revert reason if available
    if revertErr, ok := err.(vm.RevertError); ok {
        receipt.RevertReason = revertErr.ErrorData()
    }
    
    // Still increment the nonce even on failure
    state.SetNonce(tx.From(), state.GetNonce(tx.From())+1)
    
    return receipt
}
```

### Events and Logs

Recording and emitting event data:

1. **Log Structure**:
   - Contract address (source)
   - Up to 4 indexed topics
   - Data payload (unindexed parameters)
   - Block and transaction context

2. **Event Creation**:
   - LOG0-LOG4 opcodes in the PVM
   - Solidity events mapping to logs
   - Indexed vs. unindexed parameters
   - Gas cost based on data size

3. **Log Storage and Indexing**:
   - Stored in transaction receipt
   - Indexed in bloom filter
   - Queryable via JSON-RPC
   - Events not accessible from contracts

4. **Implementation Example**:

```go
// Process LOG opcodes
func opLog(pc *uint64, evm *EVM, contract *Contract, memory *Memory, stack *Stack) ([]byte, error) {
    mStart, mSize := stack.pop(), stack.pop()
    
    // Extract topics based on the specific LOG opcode (LOG0...LOG4)
    topics := make([]common.Hash, 0, int(inst.opcode-LOG0))
    for i := 0; i < int(inst.opcode-LOG0); i++ {
        topics = append(topics, common.BigToHash(stack.pop()))
    }
    
    // Copy data from memory
    data := memory.GetCopy(int64(mStart.Uint64()), int64(mSize.Uint64()))
    
    // Create the log
    evm.StateDB.AddLog(&types.Log{
        Address: contract.Address(),
        Topics:  topics,
        Data:    data,
        // Block and transaction context is added later
    })
    
    return nil, nil
}
```

## Advanced Execution Features

### Precompiled Contracts

Built-in functions for efficient operations:

1. **Execution Process**:
   - Address-based dispatch (0x01-0x09)
   - Input data validation
   - Native implementation execution
   - Gas cost calculation based on input
   - Result formatting and return

2. **Common Precompiles**:
   - ECRECOVER (0x01): Signature recovery
   - SHA256 (0x02): SHA-256 hash
   - RIPEMD160 (0x03): RIPEMD-160 hash
   - IDENTITY (0x04): Data copy
   - MODEXP (0x05): Modular exponentiation
   - BN256 operations (0x06-0x08): Elliptic curve operations

3. **Implementation Example**:

```go
// Precompiled contract interface
type PrecompiledContract interface {
    RequiredGas(input []byte) uint64
    Run(input []byte) ([]byte, error)
}

// Example: ECRECOVER precompiled contract
type ecrecover struct{}

func (e *ecrecover) RequiredGas(input []byte) uint64 {
    return params.EcrecoverGas // Fixed cost of 3000 gas
}

func (e *ecrecover) Run(input []byte) ([]byte, error) {
    // Input must be at least 128 bytes
    if len(input) < 128 {
        return nil, nil
    }
    
    // Extract components from input
    hash := input[:32]
    v := input[32:64]
    r := input[64:96]
    s := input[96:128]
    
    // Convert to appropriate formats
    vByte := byte(new(big.Int).SetBytes(v).Uint64())
    if vByte < 27 {
        vByte += 27
    }
    
    // Recover public key
    pubKey, err := crypto.Ecrecover(hash, append(append(r, s...), vByte))
    if err != nil {
        return nil, nil
    }
    
    // Return address (last 20 bytes of keccak256 hash of public key)
    return common.LeftPadBytes(
        crypto.Keccak256(pubKey[1:])[12:], 
        32,
    ), nil
}
```

### Access Lists

Pre-declaring state accesses for gas optimization:

1. **Access List Structure**:
   - List of addresses and storage keys
   - Declared in transaction (EIP-2930, EIP-1559)
   - Pre-warms caches for listed items
   - Reduces gas cost for first access

2. **Gas Benefits**:
   - Cold address access: 2600 → 100 gas
   - Cold storage access: 2100 → 100 gas
   - Cost to add to access list: 2400 per address + 1900 per storage key

3. **Implementation Example**:

```go
// Apply access list to warm caches
func applyAccessList(evm *EVM, accessList types.AccessList) {
    for _, access := range accessList {
        // Mark address as warm
        evm.StateDB.AddAddressToAccessList(access.Address)
        
        // Mark storage slots as warm
        for _, key := range access.StorageKeys {
            evm.StateDB.AddSlotToAccessList(access.Address, key)
        }
    }
}
```

### EVM Tracing

Detailed execution monitoring:

1. **Trace Types**:
   - Call tracer: Records call structure and return values
   - Structure logger: Records each operation with stack and memory
   - JavaScript tracer: Customizable JS-based tracing
   - 4byte tracer: Collects function selectors

2. **Trace Data Contents**:
   - Opcode sequence
   - Stack state at each step
   - Memory state at key points
   - Storage modifications
   - Gas consumption

3. **Implementation Example**:

```go
// Structure logger for EVM tracing
type StructLog struct {
    Pc            uint64                    `json:"pc"`
    Op            string                    `json:"op"`
    Gas           uint64                    `json:"gas"`
    GasCost       uint64                    `json:"gasCost"`
    Depth         int                       `json:"depth"`
    Error         error                     `json:"error,omitempty"`
    Stack         []string                  `json:"stack"`
    Memory        []string                  `json:"memory"`
    Storage       map[string]string         `json:"storage"`
    RefundCounter uint64                    `json:"refund"`
}

// Tracer implementation for EVM
type StructLogger struct {
    logs          []StructLog
    storage       map[common.Address]map[string]string
    output        []byte
    err           error
    gasLimit      uint64
    usedGas       uint64
    currentGas    uint64
}

func (l *StructLogger) CaptureState(pc uint64, op vm.OpCode, gas, cost uint64, scope *vm.ScopeContext, depth int, err error) {
    // Create log entry
    log := StructLog{
        Pc:            pc,
        Op:            op.String(),
        Gas:           gas,
        GasCost:       cost,
        Depth:         depth,
        Error:         err,
    }
    
    // Capture stack
    log.Stack = make([]string, len(scope.Stack))
    for i, item := range scope.Stack {
        log.Stack[i] = item.String()
    }
    
    // Capture memory
    log.Memory = make([]string, (len(scope.Memory)+31)/32)
    for i := 0; i < len(log.Memory); i++ {
        offset := i * 32
        log.Memory[i] = hex.EncodeToString(scope.Memory[offset:min(offset+32, len(scope.Memory))])
    }
    
    // Add log to collection
    l.logs = append(l.logs, log)
}
```

### Cross-Contract Calls

How contracts interact with each other:

1. **Call Types**:
   - CALL: Regular call with separate context
   - STATICCALL: Read-only call (EIP-214)
   - DELEGATECALL: Call preserving msg.sender and msg.value
   - CALLCODE: Legacy call (deprecated)

2. **Context Handling**:
   - Call depth limit (1024)
   - Gas forwarding and sub-allocation
   - Value transfers during calls
   - Return data handling

3. **Implementation Example**:

```go
// Process CALL opcode
func opCall(pc *uint64, evm *EVM, contract *Contract, memory *Memory, stack *Stack) ([]byte, error) {
    // Extract arguments from stack
    gas := stack.pop().Uint64()
    addr := common.BigToAddress(stack.pop())
    value := stack.pop()
    inOffset := stack.pop().Uint64()
    inSize := stack.pop().Uint64()
    retOffset := stack.pop().Uint64()
    retSize := stack.pop().Uint64()
    
    // Get input data from memory
    input := memory.GetPtr(int64(inOffset), int64(inSize))
    
    // Apply gas rules
    gasTemp, err := callGas(evm.gasTable, evm.Context.BlockNumber, gas, contract.Gas, stack)
    if err != nil {
        return nil, err
    }
    
    if !contract.UseGas(gasTemp) {
        return nil, ErrOutOfGas
    }
    
    // Make sure we have enough balance
    if value.Sign() != 0 && !evm.Context.CanTransfer(evm.StateDB, contract.Address(), value) {
        return nil, ErrInsufficientBalance
    }
    
    // Execute the call
    ret, returnGas, err := evm.Call(
        contract,
        addr,
        input,
        gas,
        value,
    )
    
    // Handle return value
    if err != nil {
        stack.push(new(big.Int)) // push 0 for failure
    } else {
        stack.push(big.NewInt(1)) // push 1 for success
        
        // Return unused gas
        contract.Gas += returnGas
        
        // Copy output to memory
        memory.Set(retOffset, retSize, ret)
    }
    
    return ret, nil
}
```

## Execution in the Network

### Block-Level Execution

How transactions are executed within blocks:

1. **Execution Sequence**:
   - Transactions ordered by gas price and nonce
   - Sequential execution of each transaction
   - Cumulative gas tracking against block gas limit
   - State updates after each transaction
   - Block finalization after all transactions

2. **Block Processing Steps**:
   - Validate block header and structure
   - Execute each transaction in order
   - Apply mining/validator rewards
   - Calculate state, receipt, and transaction roots
   - Finalize and commit block state

3. **Implementation Example**:

```go
// Process all transactions in a block
func ProcessBlock(block *types.Block, stateDB *state.StateDB) (*state.StateDB, []*types.Receipt, error) {
    // Validate block header
    if err := ValidateHeader(block.Header()); err != nil {
        return nil, nil, err
    }
    
    // Initialize EVM for this block
    blockContext := NewEVMBlockContext(block.Header(), chain, nil)
    evm := vm.NewEVM(blockContext, vm.TxContext{}, stateDB, chainConfig, vm.Config{})
    
    // Process all transactions
    receipts := make([]*types.Receipt, 0, len(block.Transactions()))
    cumulativeGasUsed := uint64(0)
    
    for i, tx := range block.Transactions() {
        // Create message from transaction
        msg, err := tx.AsMessage(types.MakeSigner(chainConfig, block.Number()), block.BaseFee())
        if err != nil {
            return nil, nil, err
        }
        
        // Create transaction context
        txContext := NewEVMTxContext(msg)
        evm.Reset(txContext)
        
        // Execute transaction
        result, err := ApplyMessage(evm, msg, new(GasPool).AddGas(block.GasLimit()))
        if err != nil {
            return nil, nil, err
        }
        
        // Update cumulative gas
        cumulativeGasUsed += result.UsedGas
        
        // Create receipt
        receipt := &types.Receipt{
            Type:              tx.Type(),
            Status:            types.ReceiptStatusSuccessful,
            CumulativeGasUsed: cumulativeGasUsed,
            Logs:              stateDB.GetLogs(tx.Hash()),
            TxHash:            tx.Hash(),
            GasUsed:           result.UsedGas,
            // ... other receipt fields
        }
        
        if result.Err != nil {
            receipt.Status = types.ReceiptStatusFailed
        }
        
        receipts = append(receipts, receipt)
    }
    
    // Apply block rewards
    ApplyReward(stateDB, block.Header())
    
    // Return updated state and receipts
    return stateDB, receipts, nil
}
```

### Validator Rewards

How block producers are compensated:

1. **Reward Components**:
   - Block reward: Fixed amount per block
   - Uncle rewards: Partial reward for uncle blocks
   - Transaction fees: Gas fees from all transactions
   - Priority fees: Tips from EIP-1559 transactions

2. **ProzChain Reward Distribution**:
   - 70% of base fee burned
   - 20% of base fee to treasury
   - 10% of base fee + 100% of priority fee to validators
   - Additional rewards for zero-knowledge circuits

3. **Implementation Example**:

```go
// Apply block rewards to validators
func ApplyReward(state *state.StateDB, header *types.Header) {
    // Calculate block reward based on protocol rules
    blockReward := CalcBlockReward(header.Number)
    
    // Apply block reward to coinbase (validator)
    state.AddBalance(header.Coinbase, blockReward)
    
    // Apply uncle rewards if any
    for i, uncle := range unclesToProcess {
        // Calculate uncle reward (depends on nephew distance)
        r := new(big.Int)
        r.Mul(blockReward, big.NewInt(8+int64(uncle.Number.Uint64())-int64(header.Number.Uint64()))
        r.Div(r, big.NewInt(8))
        
        // Apply uncle reward to uncle's coinbase
        state.AddBalance(uncle.Coinbase, r)
    }
}
```

### Parallelization Opportunities

Areas where execution can be optimized:

1. **Transaction-Level Parallelism**:
   - Independent transaction identification
   - Dependency graph construction
   - Parallel execution of non-conflicting transactions
   - Execution result merging

2. **EVM Instruction Parallelism**:
   - SIMD operations for math-heavy workloads
   - Parallel hash computation
   - Concurrent signature verification
   - Just-in-time compilation with parallel optimization

3. **Implementation Considerations**:
   - Deterministic output requirements
   - Race condition prevention
   - Overhead vs. benefit tradeoff
   - Hardware acceleration opportunities

## Special Execution Environments

### Zero-Knowledge Execution Circuits

ProzChain's privacy-preserving execution:

1. **ZK Circuit Structure**:
   - Private inputs and public inputs
   - Confidential state transitions
   - Verification interface for on-chain validation
   - Off-chain proof generation

2. **Execution Flow**:
   - Off-chain transaction preparation
   - ZK proof generation
   - On-chain proof verification
   - State update with minimal disclosure

3. **ProzChain ZK Features**:
   - Confidential transfers
   - Private state variables
   - Selective disclosure proofs
   - Auditable privacy

### Trusted Execution Environments (TEEs)

Secure hardware-backed execution:

1. **TEE Integration**:
   - Enclave-based execution
   - Remote attestation for validator verification
   - Confidentiality and integrity guarantees
   - Secure key management

2. **ProzChain TEE Applications**:
   - Confidential smart contracts
   - Protected validator operations
   - Secure random number generation
   - Privacy-preserving oracle data

3. **Security Considerations**:
   - Side-channel attack mitigation
   - Attestation verification
   - Enclave code review and validation
   - Hardware security assumptions

### Optimistic Rollups

Layer 2 execution with delayed verification:

1. **Execution Process**:
   - Off-chain transaction batching
   - Optimistic state transition posting
   - Fraud proof challenge period
   - Finalization after challenge period

2. **State Verification**:
   - Challenge submission
   - On-chain replay of disputed transaction
   - State root comparison
   - Slashing of incorrect assertions

3. **ProzChain Integration**:
   - Native rollup support
   - Cross-layer transaction execution
   - Standardized fraud proof protocol
   - Gas efficiency for verification operations

## Conclusion

Transaction execution is the core process by which ProzChain implements its computation and state transition system. The PVM provides a deterministic, secure, and gas-metered environment for executing smart contracts and processing transactions. Understanding the execution model is essential for developers who want to create efficient and secure applications on ProzChain.

Key takeaways:
- Transactions are executed in a stack-based virtual machine with precise gas accounting
- Different transaction types follow specific execution paths
- Contract interactions involve complex state manipulations and context handling
- Gas metering ensures fair compensation for computational resources
- Advanced features like precompiled contracts and access lists optimize common operations

In the next document, [State Changes](./transaction-lifecycle-state-changes.md), we'll explore in detail how transaction execution results in modifications to the blockchain's state.
