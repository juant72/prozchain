# Exceptional Conditions

## Overview

While the ideal transaction lifecycle flows smoothly from creation to finality, real-world blockchain networks must handle numerous exceptional conditions and edge cases. This document explores how ProzChain manages transaction failures, network anomalies, consensus disruptions, and other exceptional conditions that may occur during transaction processing. Understanding these scenarios is crucial for building robust applications that can gracefully handle adverse conditions and provide a reliable user experience despite the inherent complexity of distributed systems.

This document covers transaction-level failures, network-level anomalies, and recovery mechanisms, providing developers with practical guidance for designing resilient applications on ProzChain.

## Transaction-Level Exceptions

### Transaction Rejections

Reasons and handling for rejected transactions:

1. **Pre-Execution Rejections**:
   - Invalid transaction format
   - Invalid signature
   - Insufficient gas limit
   - Gas price too low
   - Nonce too low or too high

2. **Common Rejection Scenarios**:

   - **Invalid Nonce**: Transaction nonce doesn't match account's current nonce
   - **Insufficient Balance**: Account has insufficient funds for gas * gas price
   - **Gas Limit Too Low**: Transaction's gas limit is below intrinsic gas cost
   - **Gas Price Too Low**: Transaction's gas price is below mempool minimum
   - **Expired Transaction**: Transaction's validity period has passed

3. **RPC Error Responses**:

```json
// Example RPC error response for invalid nonce
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32010,
    "message": "nonce too low",
    "data": {
      "txHash": "0x123...",
      "expected": "0x5",
      "provided": "0x1"
    }
  }
}
```

4. **Client-Side Handling**:

```javascript
// Handle common transaction submission errors
async function sendTransactionWithErrorHandling(signedTx) {
  try {
    const txResponse = await provider.sendTransaction(signedTx);
    return { success: true, response: txResponse };
  } catch (error) {
    // Extract error information
    const errorCode = error.code;
    const errorMessage = error.message;
    
    // Handle specific error cases
    switch (true) {
      case errorMessage.includes('nonce too low'):
        return {
          success: false,
          error: 'NONCE_TOO_LOW',
          recommendation: 'Increase the nonce to match current account state'
        };
        
      case errorMessage.includes('insufficient funds'):
        return {
          success: false,
          error: 'INSUFFICIENT_FUNDS',
          recommendation: 'Ensure account has sufficient balance for transaction'
        };
        
      case errorMessage.includes('gas price too low'):
        return {
          success: false,
          error: 'GAS_PRICE_TOO_LOW',
          recommendation: 'Increase gas price to meet current network minimum'
        };
        
      case errorMessage.includes('already known'):
        return {
          success: false,
          error: 'DUPLICATE_TRANSACTION',
          recommendation: 'Transaction with same hash is already pending'
        };
      
      default:
        return {
          success: false,
          error: 'UNKNOWN_ERROR',
          message: errorMessage,
          code: errorCode
        };
    }
  }
}
```

### Execution Failures

Handling transactions that fail during execution:

1. **Execution Failure Types**:
   - Out of gas exceptions
   - Reverted transactions
   - Invalid instructions
   - Stack/memory errors
   - Access control failures

2. **Common Execution Errors**:
   - **Out of Gas**: Transaction execution exhausted gas limit
   - **Revert**: Contract explicitly reverted the transaction
   - **Invalid Opcode**: Transaction attempted to execute an invalid instruction
   - **Stack Overflow/Underflow**: Stack manipulation errors
   - **Static Call Violation**: Write operation during static call

3. **Revert Reason Extraction**:

```javascript
// Extract revert reason from failed transaction
async function extractRevertReason(txHash) {
  // Get transaction
  const tx = await provider.getTransaction(txHash);
  
  // Get receipt to confirm failure
  const receipt = await provider.getTransactionReceipt(txHash);
  if (!receipt || receipt.status !== 0) {
    return { failed: false };
  }
  
  try {
    // Replay transaction to get the error
    await provider.call({
      from: tx.from,
      to: tx.to,
      data: tx.data,
      value: tx.value,
      gasPrice: tx.gasPrice,
      gasLimit: tx.gasLimit
    }, receipt.blockNumber);
    
    // If we get here, replay didn't fail (unusual)
    return { failed: true, reason: "Unknown failure - replay succeeded" };
  } catch (error) {
    // Parse error to extract revert reason
    const errorData = error.data || error.error?.data;
    
    if (errorData) {
      // Standard revert reason format
      const hexReason = errorData.substring(10);
      
      try {
        // Attempt to decode as a string
        const abiCoder = new ethers.utils.AbiCoder();
        const decodedReason = abiCoder.decode(['string'], '0x' + hexReason)[0];
        return { failed: true, reason: decodedReason, rawError: errorData };
      } catch (decodeError) {
        // If decoding fails, return the raw data
        return { failed: true, reason: "Raw error data: " + errorData };
      }
    }
    
    // Generic error without data
    return { failed: true, reason: error.message };
  }
}
```

4. **Custom Error Decoding**:

```javascript
// Decode custom errors (Solidity 0.8.4+)
function decodeCustomError(errorData, errorAbi) {
  // Custom errors have a 4-byte selector
  const errorSelector = errorData.slice(0, 10);
  
  // Find matching error definition in ABI
  const errorDef = errorAbi.find(
    def => def.type === 'error' && 
           ethers.utils.id(
             `${def.name}(${def.inputs.map(i => i.type).join(',')})`
           ).slice(0, 10) === errorSelector
  );
  
  if (!errorDef) {
    return { name: 'Unknown error', args: [], selector: errorSelector };
  }
  
  // Decode error arguments
  const abiCoder = new ethers.utils.AbiCoder();
  const args = abiCoder.decode(
    errorDef.inputs.map(i => i.type),
    '0x' + errorData.slice(10)
  );
  
  // Return decoded error with name and arguments
  return {
    name: errorDef.name,
    args: args,
    formattedArgs: errorDef.inputs.reduce((obj, input, index) => {
      obj[input.name] = args[index];
      return obj;
    }, {})
  };
}
```

### Gas-Related Issues

Handling problems with gas limits and costs:

1. **Gas Estimation Failures**:
   - Inaccurate gas limit projections
   - Estimation simulation failures
   - Unexpected gas price volatility
   - Execution path changes at runtime

2. **Gas Price Spike Handling**:

```javascript
// Handle gas price spikes with retries and limits
async function sendTransactionWithGasStrategy(txRequest, options = {}) {
  const {
    maxGasPrice = ethers.utils.parseUnits("300", "gwei"),
    minGasPrice = ethers.utils.parseUnits("1", "gwei"),
    gasPriceIncreaseFactor = 1.1,
    maxAttempts = 5,
    delayBetweenAttempts = 15000, // 15 seconds
  } = options;
  
  // Get current gas price
  let currentGasPrice = await provider.getGasPrice();
  
  // Make sure gas price is within bounds
  if (currentGasPrice.lt(minGasPrice)) {
    currentGasPrice = minGasPrice;
  }
  
  let lastError;
  let attempt = 0;
  
  while (attempt < maxAttempts) {
    attempt++;
    
    try {
      // Update transaction with current gas price
      const txWithGas = {
        ...txRequest,
        gasPrice: currentGasPrice
      };
      
      // Check if gas price exceeds maximum
      if (currentGasPrice.gt(maxGasPrice)) {
        return {
          success: false,
          error: 'GAS_PRICE_EXCEEDED_MAXIMUM',
          gasPrice: ethers.utils.formatUnits(currentGasPrice, "gwei")
        };
      }
      
      // Send transaction
      const txResponse = await wallet.sendTransaction(txWithGas);
      return { success: true, transaction: txResponse };
    } catch (error) {
      lastError = error;
      
      // Check if error is gas price related
      if (error.message.includes('gas price too low') || 
          error.message.includes('replacement transaction underpriced') ||
          error.message.includes('transaction underpriced')) {
        
        // Increase gas price for next attempt
        currentGasPrice = currentGasPrice.mul(
          ethers.BigNumber.from(Math.floor(gasPriceIncreaseFactor * 100))
        ).div(100);
        
        console.log(`Increasing gas price to ${ethers.utils.formatUnits(currentGasPrice, "gwei")} gwei for attempt ${attempt+1}`);
        
        // Wait before retry
        await new Promise(resolve => setTimeout(resolve, delayBetweenAttempts));
        continue;
      }
      
      // If error is not gas related, don't retry
      return { success: false, error: error.message };
    }
  }
  
  return { 
    success: false, 
    error: 'MAX_ATTEMPTS_REACHED',
    lastError: lastError?.message
  };
}
```

3. **Dynamic Gas Limit Adjustment**:

```javascript
// Estimate gas with buffer and validation
async function estimateGasWithBuffer(txRequest) {
  // Base gas estimate
  let baseEstimate;
  try {
    baseEstimate = await provider.estimateGas(txRequest);
  } catch (error) {
    return {
      success: false,
      error: 'GAS_ESTIMATION_FAILED',
      message: error.message
    };
  }
  
  // Add buffer based on transaction complexity
  let buffer = 1.2; // Default 20% buffer
  
  // If transaction is to a contract, use larger buffer
  if (txRequest.data && txRequest.data !== '0x') {
    // Complex contract interaction may need more buffer
    if (txRequest.data.length > 1000) {
      buffer = 1.4; // 40% buffer for complex transactions
    } else {
      buffer = 1.3; // 30% buffer for regular contract interactions
    }
  }
  
  // Calculate gas limit with buffer
  const gasLimit = baseEstimate.mul(
    ethers.BigNumber.from(Math.floor(buffer * 100))
  ).div(100);
  
  // Verify gas limit is within block gas limit
  const block = await provider.getBlock('latest');
  if (gasLimit.gt(block.gasLimit)) {
    return {
      success: false,
      error: 'GAS_LIMIT_EXCEEDS_BLOCK_GAS_LIMIT',
      estimated: gasLimit.toString(),
      blockGasLimit: block.gasLimit.toString()
    };
  }
  
  return {
    success: true,
    gasLimit,
    baseEstimate: baseEstimate.toString(),
    multiplier: buffer
  };
}
```

### Nonce Management Issues

Handling problems with transaction sequencing:

1. **Nonce Gaps**:
   - Missing transactions in sequence
   - Transactions stuck due to low gas price
   - Out-of-order confirmation
   - Mempool eviction

2. **Nonce Management Strategy**:

```javascript
// Advanced nonce management with gap filling
class NonceManager {
  constructor(provider, address) {
    this.provider = provider;
    this.address = address;
    this.pendingNonces = new Map();
    this.confirmedNonces = new Set();
    this.highestConfirmedNonce = -1;
  }
  
  async initialize() {
    // Get current on-chain nonce
    this.currentNonce = await this.provider.getTransactionCount(this.address);
    this.highestConfirmedNonce = this.currentNonce - 1;
    
    // Check for any pending transactions
    await this.syncPendingTransactions();
    
    return this.currentNonce;
  }
  
  async syncPendingTransactions() {
    // Get all pending transactions for address
    const pendingTxs = await this.provider.send(
      'proz_getPendingTransactions',
      [{ from: this.address }]
    );
    
    // Clear current pending nonces
    this.pendingNonces.clear();
    
    // Add all pending transactions to tracking
    for (const tx of pendingTxs) {
      const nonce = parseInt(tx.nonce, 16);
      this.pendingNonces.set(nonce, tx.hash);
    }
    
    // Update current nonce if needed
    if (pendingTxs.length > 0) {
      const highestPendingNonce = Math.max(
        ...pendingTxs.map(tx => parseInt(tx.nonce, 16))
      );
      this.currentNonce = Math.max(this.currentNonce, highestPendingNonce + 1);
    }
  }
  
  async getNextNonce() {
    await this.syncPendingTransactions();
    return this.currentNonce;
  }
  
  async detectAndFixNonceGaps() {
    // Get all nonces from lowNonce to highNonce
    const pendingNonces = Array.from(this.pendingNonces.keys()).sort((a, b) => a - b);
    
    if (pendingNonces.length === 0) {
      return { gaps: [], fixed: [] };
    }
    
    // Find gaps in the nonce sequence
    const gaps = [];
    const expectedNonce = this.highestConfirmedNonce + 1;
    
    // Check for initial gap
    if (pendingNonces[0] > expectedNonce) {
      for (let n = expectedNonce; n < pendingNonces[0]; n++) {
        gaps.push(n);
      }
    }
    
    // Check for gaps between pending transactions
    for (let i = 0; i < pendingNonces.length - 1; i++) {
      const current = pendingNonces[i];
      const next = pendingNonces[i + 1];
      
      if (next > current + 1) {
        for (let n = current + 1; n < next; n++) {
          gaps.push(n);
        }
      }
    }
    
    // If we found gaps, try to fill them
    const fixed = [];
    for (const gapNonce of gaps) {
      try {
        // Send a zero-value self-transaction to fill the gap
        const tx = await this.submitNonceGapFiller(gapNonce);
        fixed.push({ nonce: gapNonce, hash: tx.hash });
      } catch (error) {
        console.error(`Failed to fill nonce gap ${gapNonce}:`, error);
      }
    }
    
    return { gaps, fixed };
  }
  
  async submitNonceGapFiller(nonce) {
    const tx = {
      to: this.address, // Self-transaction
      value: 0,
      nonce,
      gasPrice: await this.provider.getGasPrice(),
      gasLimit: 21000 // Simple value transfer
    };
    
    const signedTx = await this.wallet.signTransaction(tx);
    return await this.provider.sendTransaction(signedTx);
  }
  
  trackTransaction(nonce, txHash) {
    this.pendingNonces.set(nonce, txHash);
    this.currentNonce = Math.max(this.currentNonce, nonce + 1);
  }
  
  confirmTransaction(nonce) {
    this.pendingNonces.delete(nonce);
    this.confirmedNonces.add(nonce);
    
    // Update highest confirmed nonce
    if (nonce > this.highestConfirmedNonce) {
      this.highestConfirmedNonce = nonce;
    }
  }
}
```

3. **Transaction Replacement**:

```javascript
// Replace stuck transaction with higher gas price
async function replaceTransaction(txHash, gasPriceIncrease = 1.1) {
  // Get the original transaction
  const tx = await provider.getTransaction(txHash);
  
  if (!tx) {
    throw new Error('Transaction not found');
  }
  
  // Calculate new gas price (10% higher by default)
  const newGasPrice = tx.gasPrice.mul(
    ethers.BigNumber.from(Math.floor(gasPriceIncrease * 100))
  ).div(100);
  
  // Create replacement transaction
  const replacementTx = {
    from: tx.from,
    to: tx.to,
    nonce: tx.nonce,
    value: tx.value,
    data: tx.data,
    gasLimit: tx.gasLimit,
    gasPrice: newGasPrice
  };
  
  // Send replacement transaction
  return await wallet.sendTransaction(replacementTx);
}
```

## Network-Level Exceptions

### Chain Reorganizations

Handling blockchain restructuring:

1. **Reorg Types and Impact**:
   - Micro reorgs (1-2 blocks): Transaction reordering
   - Standard reorgs (3-5 blocks): Transaction inclusion changes
   - Deep reorgs (6+ blocks): Significant state reversions
   - Malicious reorgs: Targeted double-spend attempts

2. **Reorg Detection**:

```javascript
// Set up a reorganization detector
function setupReorgDetectionSystem(provider) {
  // Keep track of seen blocks by number
  const seenBlocks = new Map();
  
  // Subscribe to new block headers
  provider.on('block', async (blockNumber) => {
    // Get the block
    const block = await provider.getBlock(blockNumber);
    
    // Check if we've seen another block at this height
    if (seenBlocks.has(blockNumber)) {
      const previousBlock = seenBlocks.get(blockNumber);
      
      // If block hash is different, we have a reorg
      if (previousBlock.hash !== block.hash) {
        // Detect reorg depth
        let reorgDepth = 1;
        let currentBlock = block;
        let divergenceBlock = null;
        
        // Trace back to find common ancestor
        while (true) {
          const parentNumber = currentBlock.number - 1;
          const parent = await provider.getBlock(parentNumber);
          const previousParent = await provider.getBlock(
            previousBlock.parentHash
          );
          
          if (parent.hash === previousParent.hash) {
            divergenceBlock = parent;
            break;
          }
          
          reorgDepth++;
          currentBlock = parent;
        }
        
        // Log or handle the reorg
        console.warn(`Chain reorganization detected! Depth: ${reorgDepth}`);
        console.warn(`Previous head: ${previousBlock.hash}`);
        console.warn(`New head: ${block.hash}`);
        console.warn(`Common ancestor: ${divergenceBlock.hash} at #${divergenceBlock.number}`);
        
        // Emit reorg event
        eventEmitter.emit('chainReorganization', {
          depth: reorgDepth,
          previousHead: previousBlock,
          newHead: block,
          commonAncestor: divergenceBlock
        });
        
        // Check for affected transactions
        checkAffectedTransactions(divergenceBlock.number + 1, blockNumber);
      }
    }
    
    // Update seen blocks
    seenBlocks.set(blockNumber, block);
    
    // Prune old entries to manage memory
    const oldBlocks = [...seenBlocks.keys()].filter(
      num => num < blockNumber - 100
    );
    
    for (const oldBlock of oldBlocks) {
      seenBlocks.delete(oldBlock);
    }
  });
}

// Check for transactions affected by reorg
async function checkAffectedTransactions(fromBlock, toBlock) {
  // Get transactions from local database that were in the reorged blocks
  const potentiallyAffectedTxs = await db.transactions.where('blockNumber')
    .between(fromBlock, toBlock)
    .toArray();
  
  // For each transaction, check if it's still in the chain
  for (const tx of potentiallyAffectedTxs) {
    const receipt = await provider.getTransactionReceipt(tx.hash);
    
    if (!receipt) {
      // Transaction was removed from chain
      console.warn(`Transaction ${tx.hash} was removed in reorg!`);
      
      // Update status in database
      await db.transactions.update(tx.id, {
        status: 'reorged',
        currentBlockHeight: null,
        confirmations: 0
      });
      
      // Emit event for UI updates
      eventEmitter.emit('transactionReorged', tx);
      
      // Check mempool to see if it's pending again
      const pendingTx = await provider.getTransaction(tx.hash);
      if (pendingTx) {
        console.log(`Transaction ${tx.hash} is back in the mempool`);
      } else {
        console.warn(`Transaction ${tx.hash} not found in mempool, may need resubmission`);
      }
    } else if (receipt.blockNumber !== tx.blockNumber || 
               receipt.blockHash !== tx.blockHash) {
      // Transaction moved to a different block
      console.log(`Transaction ${tx.hash} moved from block ${tx.blockNumber} to ${receipt.blockNumber}`);
      
      // Update in database
      await db.transactions.update(tx.id, {
        blockNumber: receipt.blockNumber,
        blockHash: receipt.blockHash,
        status: 'confirmed'
      });
    }
  }
}
```

3. **Application Impact Mitigation**:
   - Wait for sufficient confirmations based on value
   - Monitor for unexpected state changes
   - Implement resubmission strategy for dropped transactions
   - Use event logs with reorg detection

### Network Partitions

Handling disruptions in network connectivity:

1. **Partition Scenarios**:
   - Regional network outages
   - Internet backbone disruptions
   - BGP route hijacking
   - Protocol-level partitions

2. **Split Network Detection**:

```javascript
// Detect potential network partitions
function monitorNetworkPartition(provider) {
  // Track recent blocks
  let recentBlocks = [];
  const expectedBlockInterval = 2000; // 2 seconds
  const blockMonitoringPeriod = 60000; // 1 minute
  
  // Subscribe to new blocks
  provider.on('block', async (blockNumber) => {
    const block = await provider.getBlock(blockNumber);
    const now = Date.now();
    
    // Add to recent blocks
    recentBlocks.push({
      number: blockNumber,
      timestamp: block.timestamp * 1000, // Convert to milliseconds
      receivedAt: now,
      hash: block.hash
    });
    
    // Prune old blocks
    recentBlocks = recentBlocks.filter(
      b => now - b.receivedAt < blockMonitoringPeriod
    );
    
    // Check if there's a partition:
    
    // 1. Blockchain timestamp irregularities
    if (recentBlocks.length > 10) {
      const timeDeltas = [];
      for (let i = 1; i < recentBlocks.length; i++) {
        timeDeltas.push(
          recentBlocks[i].timestamp - recentBlocks[i-1].timestamp
        );
      }
      
      const averageDelta = timeDeltas.reduce((sum, delta) => sum + delta, 0) / timeDeltas.length;
      const maxDeviation = Math.max(...timeDeltas.map(d => Math.abs(d - averageDelta)));
      
      if (maxDeviation > expectedBlockInterval * 5) {
        console.warn('Potential network partition detected: Irregular block intervals');
      }
    }
    
    // 2. Validator set changes or unusual patterns
    if (blockNumber % 10 === 0) {
      const validators = await getValidatorSetForBlock(blockNumber);
      const unusualActivityDetected = detectUnusualValidatorActivity(validators);
      
      if (unusualActivityDetected) {
        console.warn('Potential network partition detected: Unusual validator activity');
      }
    }
    
    // 3. Transaction flow analysis
    const txCount = block.transactions.length;
    const recentTxCounts = recentBlocks.slice(-10).map(b => getTransactionCount(b.number));
    const averageTxCount = recentTxCounts.reduce((sum, count) => sum + count, 0) / recentTxCounts.length;
    
    if (txCount < averageTxCount * 0.2 && averageTxCount > 10) {
      console.warn('Potential network partition detected: Unusual transaction volume drop');
    }
  });
  
  // Also monitor connection status
  let lastBlockTime = Date.now();
  
  // Check for stalled blocks
  setInterval(() => {
    const now = Date.now();
    if (now - lastBlockTime > expectedBlockInterval * 10) {
      console.warn(`Potential network partition: No new blocks for ${(now - lastBlockTime)/1000}s`);
      
      // Try to check alternate endpoints
      checkAlternateNetworkEndpoints();
    }
  }, expectedBlockInterval * 2);
}
```

3. **Partition Recovery**:

```javascript
// Recover from network partition
async function recoverFromPartition(provider, backupProviders) {
  // Check if our provider is on a minority fork
  const localBlock = await provider.getBlock('latest');
  
  // Compare with backup providers
  const alternativeChainHeights = await Promise.all(
    backupProviders.map(async (bp) => {
      try {
        const block = await bp.getBlock('latest');
        return {
          provider: bp,
          blockNumber: block.number,
          blockHash: block.hash
        };
      } catch (error) {
        console.error('Error checking backup provider:', error);
        return { provider: bp, blockNumber: 0, error };
      }
    })
  );
  
  // Filter out failed responses
  const validResponses = alternativeChainHeights.filter(r => !r.error);
  
  if (validResponses.length === 0) {
    console.error('All backup providers failed, cannot determine correct chain');
    return false;
  }
  
  // Find the majority chain height
  const heightCounts = {};
  validResponses.forEach(r => {
    heightCounts[r.blockNumber] = (heightCounts[r.blockNumber] || 0) + 1;
  });
  
  let majorityHeight = 0;
  let maxCount = 0;
  
  Object.entries(heightCounts).forEach(([height, count]) => {
    if (count > maxCount) {
      maxCount = count;
      majorityHeight = parseInt(height);
    }
  });
  
  // If our node is significantly behind, we're likely partitioned
  if (localBlock.number < majorityHeight - 10) {
    console.warn(`Local node appears to be partitioned: local height ${localBlock.number}, majority height ${majorityHeight}`);
    
    // Find a provider with the majority height
    const bestProvider = validResponses.find(
      r => r.blockNumber >= majorityHeight
    ).provider;
    
    // Switch to the better provider
    console.log('Switching to healthy provider');
    return bestProvider;
  }
  
  // If we're at a similar height but on a different chain, compare by hash
  if (Math.abs(localBlock.number - majorityHeight) < 10) {
    // Get a block we should agree on
    const commonAncestorNumber = Math.min(localBlock.number, majorityHeight) - 10;
    
    // Get the block at this height from both chains
    const localCommonBlock = await provider.getBlock(commonAncestorNumber);
    
    // Check if any backup provider has a different hash at this height
    let onDifferentChain = false;
    for (const bp of backupProviders) {
      try {
        const remoteBlock = await bp.getBlock(commonAncestorNumber);
        if (remoteBlock.hash !== localCommonBlock.hash) {
          onDifferentChain = true;
          console.warn(
            `Chain fork detected: local hash ${localCommonBlock.hash} vs remote hash ${remoteBlock.hash} at block ${commonAncestorNumber}`
          );
          break;
        }
      } catch (error) {
        continue;
      }
    }
    
    if (onDifferentChain) {
      // We're on a minority fork, switch providers
      const bestProvider = validResponses.sort((a, b) => b.blockNumber - a.blockNumber)[0].provider;
      console.log('Switching to provider on majority chain');
      return bestProvider;
    }
  }
  
  // We appear to be on the correct chain
  console.log('No partition detected, or we are on the majority chain');
  return null;
}
```

### Transaction Timeouts

Handling long-pending transactions:

1. **Timeout Scenarios**:
   - Network congestion delay
   - Gas price too low for inclusion
   - Nonce gaps preventing execution
   - Mempool eviction

2. **Timeout Detection and Handling**:

```javascript
// Monitor and handle transaction timeouts
class TransactionTimeoutManager {
  constructor(provider, options = {}) {
    this.provider = provider;
    
    // Configuration
    this.options = {
      pendingTimeout: 10 * 60 * 1000, // 10 minutes
      checkInterval: 30 * 1000, // 30 seconds
      minConfirmations: 1,
      ...options
    };
    
    // Tracked transactions
    this.transactions = new Map();
    
    // Start monitoring
    this.startMonitoring();
  }
  
  // Track a new transaction
  trackTransaction(txHash, metadata = {}) {
    this.transactions.set(txHash, {
      hash: txHash,
      submittedAt: Date.now(),
      timeoutAt: Date.now() + this.options.pendingTimeout,
      retryCount: 0,
      status: 'pending',
      ...metadata
    });
    
    return {
      promise: new Promise((resolve, reject) => {
        this.transactions.get(txHash).resolve = resolve;
        this.transactions.get(txHash).reject = reject;
      })
    };
  }
  
  // Start monitoring transactions
  startMonitoring() {
    this.monitorInterval = setInterval(
      () => this.checkTransactions(),
      this.options.checkInterval
    );
  }
  
  // Stop monitoring
  stopMonitoring() {
    if (this.monitorInterval) {
      clearInterval(this.monitorInterval);
    }
  }
  
  // Check all tracked transactions
  async checkTransactions() {
    const now = Date.now();
    const currentBlock = await this.provider.getBlockNumber();
    
    for (const [txHash, tx] of this.transactions.entries()) {
      // Skip already completed transactions
      if (tx.status !== 'pending') {
        continue;
      }
      
      try {
        // Check transaction receipt
        const receipt = await this.provider.getTransactionReceipt(txHash);
        
        if (receipt) {
          // Transaction confirmed
          const confirmations = currentBlock - receipt.blockNumber + 1;
          
          if (confirmations >= this.options.minConfirmations) {
            // Update status
            tx.status = receipt.status ? 'confirmed' : 'failed';
            tx.receipt = receipt;
            tx.confirmedAt = now;
            tx.confirmations = confirmations;
            
            // Resolve the promise
            if (receipt.status) {
              tx.resolve(receipt);
            } else {
              tx.reject(new Error('Transaction failed on-chain'));
            }
          }
          continue;
        }
        
        // Check if still in mempool
        const pendingTx = await this.provider.getTransaction(txHash);
        
        if (!pendingTx && tx.status === 'pending') {
          tx.mempoolMissingAt = now;
          
          // If missing from mempool for a while, mark as dropped
          if (tx.mempoolMissingAt && now - tx.mempoolMissingAt > 60000) {
            tx.status = 'dropped';
            tx.droppedAt = now;
            tx.reject(new Error('Transaction dropped from mempool'));
          }
        } else {
          // Reset missing flag if found again
          tx.mempoolMissingAt = null;
        }
        
        // Check for timeout
        if (now > tx.timeoutAt) {
          // Transaction has timed out
          if (tx.onTimeout) {
            // Custom timeout handler
            tx.onTimeout(tx, this);
          } else {
            // Default: mark as timed out
            tx.status = 'timeout';
            tx.timeoutAt = now;
            tx.reject(new Error('Transaction timed out'));
          }
        }
      } catch (error) {
        console.error(`Error checking transaction ${txHash}:`, error);
      }
    }
    
    // Clean up old completed transactions
    for (const [txHash, tx] of this.transactions.entries()) {
      if (tx.status !== 'pending' && now - (tx.confirmedAt || tx.droppedAt || tx.timeoutAt) > 3600000) {
        this.transactions.delete(txHash);
      }
    }
  }
  
  // Get transaction status
  getTransaction(txHash) {
    return this.transactions.get(txHash);
  }
  
  // Cancel a transaction by replacing it with a zero-value self-send
  async cancelTransaction(txHash, gasPriceMultiplier = 1.1) {
    const tx = this.transactions.get(txHash);
    
    if (!tx) {
      throw new Error('Transaction not tracked');
    }
    
    // Get the original transaction
    const pendingTx = await this.provider.getTransaction(txHash);
    
    if (!pendingTx) {
      throw new Error('Transaction not found in mempool');
    }
    
    // Calculate cancellation gas price (higher than original)
    const gasPrice = pendingTx.gasPrice.mul(
      ethers.BigNumber.from(Math.floor(gasPriceMultiplier * 100))
    ).div(100);
    
    // Create cancellation transaction (send 0 ETH to self with same nonce)
    const cancelTx = {
      to: pendingTx.from,
      value: 0,
      nonce: pendingTx.nonce,
      gasPrice,
      gasLimit: 21000 // Simple transfer gas limit
    };
    
    // Send cancellation transaction
    const response = await this.wallet.sendTransaction(cancelTx);
    
    // Update status
    tx.cancelAttempt = {
      hash: response.hash,
      timestamp: Date.now()
    };
    
    return response;
  }
  
  // Speed up a transaction by replacing it with higher gas price
  async speedUpTransaction(txHash, gasPriceMultiplier = 1.25) {
    const tx = this.transactions.get(txHash);
    
    if (!tx) {
      throw new Error('Transaction not tracked');
    }
    
    // Get the original transaction
    const pendingTx = await this.provider.getTransaction(txHash);
    
    if (!pendingTx) {
      throw new Error('Transaction not found in mempool');
    }
    
    // Calculate new gas price
    const gasPrice = pendingTx.gasPrice.mul(
      ethers.BigNumber.from(Math.floor(gasPriceMultiplier * 100))
    ).div(100);
    
    // Create replacement transaction with same parameters but higher gas
    const speedUpTx = {
      to: pendingTx.to,
      from: pendingTx.from,
      value: pendingTx.value,
      nonce: pendingTx.nonce,
      data: pendingTx.data,
      gasPrice,
      gasLimit: pendingTx.gasLimit
    };
    
    // Send replacement transaction
    const response = await this.wallet.sendTransaction(speedUpTx);
    
    // Update status
    tx.speedUpAttempt = {
      hash: response.hash,
      timestamp: Date.now(),
      originalGasPrice: pendingTx.gasPrice.toString(),
      newGasPrice: gasPrice.toString()
    };
    
    // Track the new transaction
    this.trackTransaction(response.hash, {
      replacementFor: txHash,
      submittedAt: Date.now(),
      timeoutAt: Date.now() + this.options.pendingTimeout
    });
    
    return response;
  }
}
```

3. **Exponential Backoff Strategy**:

```javascript
// Submit transaction with exponential backoff
async function submitWithExponentialBackoff(txRequest, options = {}) {
  const {
    maxRetries = 5,
    initialDelay = 5000,
    maxDelay = 120000,
    factor = 2,
    gasPriceIncreaseFactor = 1.1
  } = options;
  
  let attempt = 0;
  let delay = initialDelay;
  let gasPrice = await provider.getGasPrice();
  
  while (attempt < maxRetries) {
    try {
      // Update transaction with current gas price
      const tx = {
        ...txRequest,
        gasPrice
      };
      
      // Send transaction
      const response = await wallet.sendTransaction(tx);
      
      console.log(`Transaction sent successfully on attempt ${attempt + 1}`);
      return response;
    } catch (error) {
      attempt++;
      
      if (attempt >= maxRetries) {
        throw new Error(`Failed after ${maxRetries} attempts: ${error.message}`);
      }
      
      // Check if error is gas price related
      if (error.message.includes('gas price too low') ||
          error.message.includes('underpriced')) {
        // Increase gas price
        gasPrice = gasPrice.mul(
          ethers.BigNumber.from(Math.floor(gasPriceIncreaseFactor * 100))
        ).div(100);
        console.log(`Increasing gas price to ${ethers.utils.formatUnits(gasPrice, "gwei")} gwei`);
      }
      
      // Wait with exponential backoff
      console.log(`Attempt ${attempt} failed, retrying in ${delay}ms`);
      await new Promise(resolve => setTimeout(resolve, delay));
      delay = Math.min(delay * factor, maxDelay);
    }
  }
}
```

## Recovery Strategies

### Transaction Revival

Bringing back lost or stuck transactions:

1. **Mempool Revival**:
   - Resubmission of dropped transactions
   - Gas price bumping for stuck transactions
   - Preserving original transaction parameters
   - Handling nonce conflicts

2. **Example Implementation**:

```javascript
// Revive a transaction that may have been dropped
async function reviveTransaction(originalTxHash, options = {}) {
  const {
    gasPriceMultiplier = 1.2,
    useResubmit = true,
    maxAttempts = 3
  } = options;
  
  // Try to get original transaction
  const tx = await provider.getTransaction(originalTxHash);
  
  // Check transaction receipt to see if it's already mined
  const receipt = await provider.getTransactionReceipt(originalTxHash);
  if (receipt) {
    return {
      status: 'already_confirmed',
      receipt,
      hash: originalTxHash
    };
  }
  
  // If we can't find original transaction, we need the data from user
  if (!tx && !options.transactionData) {
    throw new Error('Transaction not found and no transaction data provided');
  }
  
  // Use provided data or original transaction
  const txData = options.transactionData || {
    to: tx.to,
    from: tx.from,
    value: tx.value,
    data: tx.data,
    nonce: tx.nonce,
    gasLimit: tx.gasLimit
  };
  
  // Calculate new gas price
  const baseGasPrice = tx ? tx.gasPrice : await provider.getGasPrice();
  const newGasPrice = baseGasPrice.mul(
    ethers.BigNumber.from(Math.floor(gasPriceMultiplier * 100))
  ).div(100);
  
  // Try to replace or resubmit
  if (tx && !useResubmit) {
    // Try replacement (same nonce, higher gas price)
    const replacementTx = {
      ...txData,
      gasPrice: newGasPrice
    };
    
    console.log(`Attempting to replace transaction with higher gas price: ${ethers.utils.formatUnits(newGasPrice, "gwei")} gwei`);
    
    return wallet.sendTransaction(replacementTx);
  } else {
    // Resubmit as new transaction (potentially with updated nonce)
    let nonce = txData.nonce;
    
    // If not forcing nonce, check current nonce
    if (!options.forceNonce) {
      const currentNonce = await provider.getTransactionCount(txData.from);
      
      // Only update if current nonce is higher
      if (currentNonce > nonce) {
        console.log(`Updating nonce from ${nonce} to ${currentNonce}`);
        nonce = currentNonce;
      }
    }
    
    // Create new transaction
    const newTx = {
      ...txData,
      nonce,
      gasPrice: newGasPrice
    };
    
    console.log(`Resubmitting transaction with nonce ${nonce} and gas price ${ethers.utils.formatUnits(newGasPrice, "gwei")} gwei`);
    
    return wallet.sendTransaction(newTx);
  }
}
```

### State Consistency Recovery

Ensuring application state remains valid:

1. **State Verification**:
   - On-chain state polling after transaction
   - Verification of expected state changes
   - Detection of state inconsistencies
   - Recovery from failed or partial updates

2. **Implementation Strategy**:

```javascript
// Verify and recover application state consistency
async function verifyAndRepairState(expectedState, transactionHash) {
  // Check if transaction was confirmed
  const receipt = await provider.getTransactionReceipt(transactionHash);
  
  if (!receipt) {
    console.warn('Transaction not confirmed, state may be inconsistent');
    return { consistent: false, needsRecovery: true, repaired: false };
  }
  
  // Check if transaction succeeded
  if (receipt.status === 0) {
    console.warn('Transaction failed, state may be inconsistent');
    return { consistent: false, needsRecovery: true, repaired: false };
  }
  
  // Verify on-chain state against expected state
  const currentState = await fetchCurrentState();
  const consistent = compareStates(currentState, expectedState);
  
  if (consistent) {
    return { consistent: true, needsRecovery: false };
  }
  
  console.warn('State inconsistency detected, attempting recovery');
  
  // Attempt state recovery
  try {
    // Generate repair transaction
    const repairTx = generateRepairTransaction(currentState, expectedState);
    
    // Execute repair if needed
    if (repairTx) {
      const repairReceipt = await wallet.sendTransaction(repairTx);
      await provider.waitForTransaction(repairReceipt.hash);
      
      // Verify repair was successful
      const repairedState = await fetchCurrentState();
      const repaired = compareStates(repairedState, expectedState);
      
      return { 
        consistent: repaired, 
        needsRecovery: true, 
        repaired, 
        repairTxHash: repairReceipt.hash 
      };
    }
    
    return { consistent: false, needsRecovery: true, repaired: false };
  } catch (error) {
    console.error('State recovery failed:', error);
    return { 
      consistent: false, 
      needsRecovery: true, 
      repaired: false, 
      error: error.message 
    };
  }
}

// Compare states to identify inconsistencies
function compareStates(currentState, expectedState) {
  // Implement comparison logic based on your state model
  // Returns true if states match, false otherwise
}

// Generate transaction to repair inconsistent state
function generateRepairTransaction(currentState, expectedState) {
  // Identify differences
  const differences = findStateDifferences(currentState, expectedState);
  
  // If no differences or differences can't be fixed, return null
  if (differences.length === 0 || !differences.some(d => d.fixable)) {
    return null;
  }
  
  // Create repair transaction
  // This will depend on your specific contract and state model
  const repairTx = {
    to: CONTRACT_ADDRESS,
    data: encodeRepairFunction(differences),
    gasLimit: 200000, // Higher gas limit for repair transactions
  };
  
  return repairTx;
}
```

### Circuit Breakers

Safety mechanisms for exceptional conditions:

1. **Circuit Breaker Patterns**:
   - Automatic transaction pausing on error rates
   - Gradual service degradation
   - Manual override capabilities
   - Recovery procedures

2. **Implementation Example**:

```javascript
// Circuit breaker for transaction submission
class TransactionCircuitBreaker {
  constructor(options = {}) {
    // Configuration
    this.options = {
      errorThreshold: 3,        // Number of errors before tripping
      resetTimeout: 60000,      // Time before auto-reset (1 minute)
      halfOpenLimit: 1,         // Transactions to try in half-open state
      monitorWindow: 30000,     // Time window for error counting (30 seconds)
      ...options
    };
    
    // State
    this.state = 'CLOSED';      // CLOSED, OPEN, HALF_OPEN
    this.errors = [];           // Recent errors
    this.lastStateChange = Date.now();
    this.halfOpenAttempts = 0;
  }
  
  // Check if circuit breaker allows transactions
  canSendTransaction() {
    this._updateState();
    return this.state !== 'OPEN';
  }
  
  // Record successful transaction
  recordSuccess() {
    if (this.state === 'HALF_OPEN') {
      // Reset circuit breaker on success in half-open state
      this.state = 'CLOSED';
      this.lastStateChange = Date.now();
      this.errors = [];
      this.halfOpenAttempts = 0;
      console.log('Circuit breaker reset to CLOSED after successful transaction');
    }
    
    // In CLOSED state, we just continue
  }
  
  // Record failed transaction
  recordFailure(error) {
    // Add error to tracking
    this.errors.push({
      timestamp: Date.now(),
      error: error
    });
    
    if (this.state === 'HALF_OPEN') {
      // In half-open state, one failure trips circuit back open
      this.state = 'OPEN';
      this.lastStateChange = Date.now();
      this.halfOpenAttempts = 0;
      console.log('Circuit breaker tripped OPEN after failure in HALF_OPEN state');
    } else if (this.state === 'CLOSED') {
      // Remove old errors outside monitoring window
      const now = Date.now();
      this.errors = this.errors.filter(
        e => now - e.timestamp < this.options.monitorWindow
      );
      
      // Check if we're over threshold
      if (this.errors.length >= this.options.errorThreshold) {
        this.state = 'OPEN';
        this.lastStateChange = Date.now();
        console.log(`Circuit breaker tripped OPEN after ${this.errors.length} errors`);
      }
    }
    
    // In OPEN state, track error but don't change state
  }
  
  // Use the circuit breaker to wrap a transaction
  async executeTransaction(txFunc) {
    // Check if we can send
    if (!this.canSendTransaction()) {
      throw new Error('Circuit breaker is OPEN, transaction rejected');
    }
    
    try {
      // Try to execute transaction
      const result = await txFunc();
      
      // Record success
      this.recordSuccess();
      
      return result;
    } catch (error) {
      // Record failure
      this.recordFailure(error);
      
      // Rethrow for caller to handle
      throw error;
    }
  }
  
  // Reset circuit breaker manually
  reset() {
    this.state = 'CLOSED';
    this.lastStateChange = Date.now();
    this.errors = [];
    this.halfOpenAttempts = 0;
    console.log('Circuit breaker manually reset to CLOSED');
  }
  
  // Trip circuit breaker manually
  trip() {
    this.state = 'OPEN';
    this.lastStateChange = Date.now();
    console.log('Circuit breaker manually tripped to OPEN');
  }
  
  // Update state based on timers
  _updateState() {
    const now = Date.now();
    
    if (this.state === 'OPEN') {
      // Check if we've waited long enough to try again
      if (now - this.lastStateChange >= this.options.resetTimeout) {
        this.state = 'HALF_OPEN';
        this.lastStateChange = now;
        this.halfOpenAttempts = 0;
        console.log('Circuit breaker moved to HALF_OPEN after timeout');
      }
    } else if (this.state === 'HALF_OPEN') {
      // If we've tried enough transactions in half-open state without success,
      // go back to open
      if (this.halfOpenAttempts >= this.options.halfOpenLimit) {
        this.state = 'OPEN';
        this.lastStateChange = now;
        console.log('Circuit breaker returned to OPEN after too many HALF_OPEN attempts');
      }
    }
  }
}

// Usage example
const circuitBreaker = new TransactionCircuitBreaker();

async function sendTransactionWithCircuitBreaker(txData) {
  return circuitBreaker.executeTransaction(async () => {
    // The actual transaction logic
    const tx = await wallet.sendTransaction(txData);
    await provider.waitForTransaction(tx.hash);
    return tx;
  });
}
```

## Best Practices for Exception Handling

### Robust Transaction Management

Guidelines for resilient transaction processing:

1. **Transaction Lifecycle Monitoring**:
   - Implement full lifecycle tracking
   - Watch for transaction status changes
   - Monitor mempool presence
   - Set up appropriate timeouts

2. **Retry Strategies**:
   - Exponential backoff for network issues
   - Progressive gas price increases
   - Nonce management for order-dependent transactions
   - Failed transaction analysis before resubmission

3. **Graceful Degradation**:
   - Provide feedback on transaction status
   - Implement fallback mechanisms
   - Manage user expectations during network issues
   - Offer alternative paths for critical operations

### User Experience Considerations

Making exceptional conditions transparent to users:

1. **Error Communication**:
   - Clear, non-technical error messages
   - Actionable suggestions for resolution
   - Progress updates during recovery
   - Confidence indicators for transaction status

2. **Transaction Monitoring UI**:
   - Real-time status tracking
   - Visual indicators of transaction state
   - Retry/cancel options for pending transactions
   - Resubmission interfaces for failed transactions

3. **Recovery Options**:
   - Self-service transaction acceleration
   - Guided troubleshooting for common issues
   - Automated recovery for typical scenarios
   - Support escalation for complex failures

## Conclusion

Exceptional conditions are inevitable in distributed blockchain networks, and ProzChain's transaction lifecycle includes comprehensive mechanisms for detecting, managing, and recovering from these situations. By understanding these exceptional conditions and implementing robust error handling strategies, developers can build resilient applications that provide a reliable user experience even when the network faces challenges.

Key practices for handling exceptional conditions include:
- Implementing proper transaction monitoring and timeout management
- Using appropriate retry strategies with exponential backoff
- Providing clear feedback to users during exceptional conditions
- Designing systems with graceful degradation capabilities
- Utilizing circuit breakers and other safety mechanisms

By anticipating exceptions and planning for recovery, applications built on ProzChain can maintain high availability and reliability, enhancing the overall user experience and building trust in blockchain-based services.
