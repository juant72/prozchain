# Performance Testing

## Overview

Performance testing is crucial for blockchain applications, where efficiency directly impacts costs and user experience. This chapter explores techniques for measuring and optimizing the performance of ProzChain applications, with a focus on gas usage, transaction throughput, and system scalability.

Unlike traditional applications where performance might be a secondary concern, blockchain applications must be optimized from the ground up due to the inherent constraints of the blockchain environment. Understanding and implementing performance testing methodologies helps ensure that your applications are cost-effective and responsive.

## Gas Optimization Testing

### Understanding Gas Costs

Gas is the computational cost unit for operations on the Ethereum Virtual Machine (EVM) and similar platforms. Every operation in a smart contract consumes a specific amount of gas, which translates to direct costs for users.

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Gas Usage Tests", function() {
  let contract;
  let deployer;
  
  before(async function() {
    [deployer] = await ethers.getSigners();
    const Contract = await ethers.getContractFactory("YourContract");
    contract = await Contract.deploy();
    await contract.deployed();
  });

  it("measures gas usage for a function call", async function() {
    // Estimate gas usage
    const estimatedGas = await contract.estimateGas.yourFunction(param1, param2);
    console.log(`Estimated gas: ${estimatedGas.toString()}`);
    
    // Execute function and get actual gas used
    const tx = await contract.yourFunction(param1, param2);
    const receipt = await tx.wait();
    
    console.log(`Actual gas used: ${receipt.gasUsed.toString()}`);
    
    // Optional: Assert maximum gas usage
    expect(receipt.gasUsed).to.be.lte(maxGasLimit);
  });
});
```

### Comparing Implementation Alternatives

Testing different implementation approaches to find the most gas-efficient solution:

```javascript
describe("Implementation Comparison", function() {
  let contractV1, contractV2;
  
  before(async function() {
    const ContractV1 = await ethers.getContractFactory("ContractImplementationV1");
    contractV1 = await ContractV1.deploy();
    
    const ContractV2 = await ethers.getContractFactory("ContractImplementationV2");
    contractV2 = await ContractV2.deploy();
  });
  
  it("compares gas usage between implementations", async function() {
    // Common test parameters
    const param1 = 100;
    const param2 = "0x1234567890123456789012345678901234567890";
    
    // Test first implementation
    const tx1 = await contractV1.processData(param1, param2);
    const receipt1 = await tx1.wait();
    
    // Test second implementation
    const tx2 = await contractV2.processData(param1, param2);
    const receipt2 = await tx2.wait();
    
    // Compare results
    console.log(`Implementation V1 gas used: ${receipt1.gasUsed.toString()}`);
    console.log(`Implementation V2 gas used: ${receipt2.gasUsed.toString()}`);
    
    const improvement = receipt1.gasUsed.sub(receipt2.gasUsed);
    const percentImprovement = improvement.mul(100).div(receipt1.gasUsed);
    
    console.log(`Gas savings: ${improvement} (${percentImprovement}%)`);
    
    // Assert that V2 is more efficient
    expect(receipt2.gasUsed).to.be.lt(receipt1.gasUsed);
  });
});
```

### Automated Gas Benchmarking

Setting up a gas reporter and automated benchmark testing:

```javascript
// In hardhat.config.js
module.exports = {
  // ...other config
  gasReporter: {
    enabled: (process.env.REPORT_GAS) ? true : false,
    currency: 'USD',
    gasPrice: 100, // in gwei
    coinmarketcap: process.env.COINMARKETCAP_API_KEY,
    excludeContracts: ['mocks/'],
    src: './contracts',
  }
};
```

Creating a gas benchmarking script:

```javascript
// scripts/gas-benchmark.js
const fs = require('fs');
const { ethers } = require('hardhat');

async function main() {
  // Load previous benchmarks if they exist
  let previousBenchmarks = {};
  const benchmarkFile = './gas-benchmarks.json';
  if (fs.existsSync(benchmarkFile)) {
    previousBenchmarks = JSON.parse(fs.readFileSync(benchmarkFile));
  }
  
  // Current benchmarks
  const benchmarks = {
    timestamp: new Date().toISOString(),
    contracts: {},
    comparisons: []
  };
  
  // Deploy contracts to test
  const contractFactory = await ethers.getContractFactory("YourContract");
  const contract = await contractFactory.deploy();
  await contract.deployed();
  
  // Define test scenarios
  const scenarios = [
    { name: "simple_transfer", args: [ethers.utils.parseEther("1.0")] },
    { name: "complex_operation", args: [100, "0x1234"] },
    // Add more scenarios as needed
  ];
  
  // Run benchmark for each scenario
  for (const scenario of scenarios) {
    const tx = await contract.functionToTest(...scenario.args);
    const receipt = await tx.wait();
    
    // Store results
    benchmarks.contracts["YourContract"] = benchmarks.contracts["YourContract"] || {};
    benchmarks.contracts["YourContract"][scenario.name] = {
      gasUsed: receipt.gasUsed.toString(),
      args: scenario.args.map(a => a.toString())
    };
    
    // Compare with previous benchmark if it exists
    if (previousBenchmarks.contracts?.["YourContract"]?.[scenario.name]) {
      const previous = previousBenchmarks.contracts["YourContract"][scenario.name];
      const previousGas = ethers.BigNumber.from(previous.gasUsed);
      const currentGas = receipt.gasUsed;
      const change = currentGas.sub(previousGas);
      const percentChange = change.mul(100).div(previousGas);
      
      benchmarks.comparisons.push({
        contract: "YourContract",
        scenario: scenario.name,
        previousGas: previousGas.toString(),
        currentGas: currentGas.toString(),
        change: change.toString(),
        percentChange: percentChange.toString()
      });
      
      // Log the comparison
      const changeSymbol = change.gt(0) ? '▲' : change.lt(0) ? '▼' : '–';
      console.log(
        `${scenario.name}: ${currentGas.toString()} gas ` + 
        `${changeSymbol} ${Math.abs(percentChange)}% ` +
        `(${change.toString()})`
      );
    }
  }
  
  // Save benchmark results
  fs.writeFileSync(benchmarkFile, JSON.stringify(benchmarks, null, 2));
  
  console.log(`Benchmark results saved to ${benchmarkFile}`);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
```

### Gas Optimization Techniques

Testing common gas optimization techniques:

```javascript
describe("Gas Optimization Techniques", function() {
  let factory;
  
  before(async function() {
    factory = await ethers.getContractFactory("OptimizationTest");
  });
  
  it("tests storage vs. memory usage", async function() {
    const contract = await factory.deploy();
    
    // Test using storage
    const txStorage = await contract.processWithStorage([1, 2, 3, 4, 5]);
    const receiptStorage = await txStorage.wait();
    
    // Test using memory
    const txMemory = await contract.processWithMemory([1, 2, 3, 4, 5]);
    const receiptMemory = await txMemory.wait();
    
    console.log(`Storage gas used: ${receiptStorage.gasUsed.toString()}`);
    console.log(`Memory gas used: ${receiptMemory.gasUsed.toString()}`);
    
    // Memory should be more efficient for read-only operations
    expect(receiptMemory.gasUsed).to.be.lt(receiptStorage.gasUsed);
  });
  
  it("tests uint size optimization", async function() {
    const contract = await factory.deploy();
    
    // Test with uint256
    const txUint256 = await contract.storeWithUint256(100);
    const receiptUint256 = await txUint256.wait();
    
    // Test with uint128
    const txUint128 = await contract.storeWithUint128(100);
    const receiptUint128 = await txUint128.wait();
    
    // Test with uint8
    const txUint8 = await contract.storeWithUint8(100);
    const receiptUint8 = await txUint8.wait();
    
    console.log(`uint256 gas: ${receiptUint256.gasUsed.toString()}`);
    console.log(`uint128 gas: ${receiptUint128.gasUsed.toString()}`);
    console.log(`uint8 gas: ${receiptUint8.gasUsed.toString()}`);
    
    // In many cases, smaller uint types can save gas
    // but not always - results may vary based on compiler optimizations
  });
  
  it("tests packed struct optimization", async function() {
    const contract = await factory.deploy();
    
    // Test with unpacked struct
    const txUnpacked = await contract.storeUnpackedStruct(1, 2, true);
    const receiptUnpacked = await txUnpacked.wait();
    
    // Test with packed struct
    const txPacked = await contract.storePackedStruct(1, 2, true);
    const receiptPacked = await txPacked.wait();
    
    console.log(`Unpacked struct gas: ${receiptUnpacked.gasUsed.toString()}`);
    console.log(`Packed struct gas: ${receiptPacked.gasUsed.toString()}`);
    
    // Packed structs should use less gas due to storage optimization
    expect(receiptPacked.gasUsed).to.be.lt(receiptUnpacked.gasUsed);
  });
});
```

## Transaction Throughput Testing

### Single Transaction Performance

Measuring the performance of individual transactions:

```javascript
describe("Transaction Performance", function() {
  let contract;
  let deployer;
  
  before(async function() {
    [deployer] = await ethers.getSigners();
    const Contract = await ethers.getContractFactory("PerformanceTest");
    contract = await Contract.deploy();
  });
  
  it("measures transaction confirmation time", async function() {
    const startTime = Date.now();
    
    // Execute transaction
    const tx = await contract.performOperation();
    const receipt = await tx.wait();
    
    const endTime = Date.now();
    const confirmationTime = endTime - startTime;
    
    console.log(`Transaction confirmation time: ${confirmationTime}ms`);
    console.log(`Block number: ${receipt.blockNumber}`);
    console.log(`Gas used: ${receipt.gasUsed.toString()}`);
    
    // Optional assertion on max confirmation time
    // This is network-dependent, so adjust based on your test environment
    expect(confirmationTime).to.be.lte(5000); // 5 seconds
  });
  
  it("measures transaction execution time via events", async function() {
    // Function that emits events at start and end of execution
    const tx = await contract.timedOperation();
    const receipt = await tx.wait();
    
    // Extract timestamps from events
    const startEvent = receipt.events.find(e => e.event === 'OperationStarted');
    const endEvent = receipt.events.find(e => e.event === 'OperationCompleted');
    
    const startTime = startEvent.args.timestamp;
    const endTime = endEvent.args.timestamp;
    const executionTime = endTime.sub(startTime).toNumber();
    
    console.log(`On-chain execution time: ${executionTime} seconds`);
    
    // For intensive operations, execution time should be within reasonable limits
    expect(executionTime).to.be.lte(3); // 3 seconds max
  });
});
```

### Batch Processing Performance

Testing the performance of batch operations:

```javascript
describe("Batch Processing Performance", function() {
  let contract;
  
  before(async function() {
    const Contract = await ethers.getContractFactory("BatchProcessor");
    contract = await Contract.deploy();
  });
  
  it("compares individual vs. batch processing performance", async function() {
    const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Process items individually
    const startTimeIndividual = Date.now();
    let totalGasIndividual = ethers.BigNumber.from(0);
    
    for (const item of items) {
      const tx = await contract.processSingle(item);
      const receipt = await tx.wait();
      totalGasIndividual = totalGasIndividual.add(receipt.gasUsed);
    }
    
    const endTimeIndividual = Date.now();
    
    // Process items in batch
    const startTimeBatch = Date.now();
    const txBatch = await contract.processBatch(items);
    const receiptBatch = await txBatch.wait();
    const endTimeBatch = Date.now();
    
    // Calculate metrics
    const timeIndividual = endTimeIndividual - startTimeIndividual;
    const timeBatch = endTimeBatch - startTimeBatch;
    const gasBatch = receiptBatch.gasUsed;
    
    // Log results
    console.log(`Individual processing: ${timeIndividual}ms, ${totalGasIndividual.toString()} gas`);
    console.log(`Batch processing: ${timeBatch}ms, ${gasBatch.toString()} gas`);
    
    // Calculate savings
    const timeSavingsPercent = 100 * (timeIndividual - timeBatch) / timeIndividual;
    const gasSavingsPercent = 100 * (totalGasIndividual.sub(gasBatch)).div(totalGasIndividual);
    
    console.log(`Time savings: ${timeSavingsPercent.toFixed(2)}%`);
    console.log(`Gas savings: ${gasSavingsPercent.toString()}%`);
    
    // Batch processing should be more efficient
    expect(timeBatch).to.be.lt(timeIndividual);
    expect(gasBatch).to.be.lt(totalGasIndividual);
  });
  
  it("determines optimal batch size", async function() {
    // Test different batch sizes to find optimal performance
    const batchSizes = [5, 10, 20, 50, 100];
    const results = [];
    
    for (const size of batchSizes) {
      // Generate test data
      const items = Array.from({ length: size }, (_, i) => i + 1);
      
      // Process batch
      const startTime = Date.now();
      const tx = await contract.processBatch(items);
      const receipt = await tx.wait();
      const endTime = Date.now();
      
      // Calculate metrics
      const processingTime = endTime - startTime;
      const gasUsed = receipt.gasUsed;
      const gasPerItem = gasUsed.div(size);
      
      results.push({
        batchSize: size,
        processingTime,
        gasUsed: gasUsed.toString(),
        gasPerItem: gasPerItem.toString()
      });
      
      console.log(`Batch size ${size}: ${processingTime}ms, ${gasUsed.toString()} gas, ${gasPerItem.toString()} gas/item`);
    }
    
    // Find batch size with lowest gas per item
    const optimalBatch = results.reduce((prev, curr) => {
      return ethers.BigNumber.from(prev.gasPerItem).lt(curr.gasPerItem) ? prev : curr;
    });
    
    console.log(`Optimal batch size: ${optimalBatch.batchSize} (${optimalBatch.gasPerItem} gas/item)`);
  });
});
```

### Transaction Queue Performance

Testing performance under high transaction volume:

```javascript
describe("Transaction Queue Performance", function() {
  let contract;
  let accounts;
  
  before(async function() {
    accounts = await ethers.getSigners();
    const Contract = await ethers.getContractFactory("QueueTest");
    contract = await Contract.deploy();
  });
  
  it("measures performance with concurrent transactions", async function() {
    // Skip test if not using a suitable network
    if (network.name !== 'hardhat' && network.name !== 'localhost') {
      console.log('Skipping concurrent transaction test on non-local network');
      this.skip();
    }
    
    const numTransactions = 10;
    const startTime = Date.now();
    
    // Submit multiple transactions concurrently
    const promises = [];
    for (let i = 0; i < numTransactions; i++) {
      const value = i * 100;
      promises.push(
        accounts[i % accounts.length].sendTransaction({
          to: contract.address,
          value: ethers.utils.parseEther("0.01"),
          data: contract.interface.encodeFunctionData("recordValue", [value])
        })
      );
    }
    
    // Wait for all transactions to be mined
    const txs = await Promise.all(promises);
    const receipts = await Promise.all(txs.map(tx => tx.wait()));
    
    const endTime = Date.now();
    const totalTime = endTime - startTime;
    
    // Calculate metrics
    const txsPerSecond = (numTransactions / totalTime) * 1000;
    const avgGasUsed = receipts
      .reduce((sum, r) => sum.add(r.gasUsed), ethers.BigNumber.from(0))
      .div(numTransactions);
    
    // Log results
    console.log(`Processed ${numTransactions} transactions in ${totalTime}ms`);
    console.log(`Throughput: ${txsPerSecond.toFixed(2)} tx/s`);
    console.log(`Average gas used: ${avgGasUsed.toString()}`);
    
    // Verify all transactions were processed
    for (let i = 0; i < numTransactions; i++) {
      const value = await contract.values(i);
      expect(value).to.equal(i * 100);
    }
  });
});
```

## Load and Stress Testing

### Setup for Load Testing

Configuring a load testing environment:

```javascript
const { ethers } = require("hardhat");
const { Worker, isMainThread, parentPort, workerData } = require('worker_threads');
const path = require('path');
const fs = require('fs');

async function runLoadTest(config) {
  const {
    numUsers,
    requestsPerUser,
    contractAddress,
    durationSeconds,
    rpcUrl
  } = config;
  
  console.log(`Starting load test with ${numUsers} users, ${requestsPerUser} requests each`);
  
  // Create custom provider for load testing
  const provider = new ethers.providers.JsonRpcProvider(rpcUrl);
  
  // Generate user wallets
  const wallets = [];
  const masterWallet = new ethers.Wallet(process.env.PRIVATE_KEY, provider);
  
  for (let i = 0; i < numUsers; i++) {
    const wallet = ethers.Wallet.createRandom().connect(provider);
    wallets.push(wallet);
    
    // Fund test wallet
    await masterWallet.sendTransaction({
      to: wallet.address,
      value: ethers.utils.parseEther("0.1")
    });
  }
  
  // Create contract interface
  const Contract = await ethers.getContractFactory("YourContract");
  const contractAbi = Contract.interface;
  
  // Start load testing with worker threads
  const workers = [];
  const workerCount = Math.min(numUsers, require('os').cpus().length);
  const usersPerWorker = Math.ceil(numUsers / workerCount);
  
  // Metrics
  let totalRequests = 0;
  let successfulRequests = 0;
  let failedRequests = 0;
  let totalLatency = 0;
  
  for (let i = 0; i < workerCount; i++) {
    const startUserIndex = i * usersPerWorker;
    const endUserIndex = Math.min(startUserIndex + usersPerWorker, numUsers);
    const workerWallets = wallets.slice(startUserIndex, endUserIndex);
    
    const worker = new Worker(path.join(__dirname, 'load-test-worker.js'), {
      workerData: {
        wallets: workerWallets.map(w => w.privateKey),
        requestsPerUser,
        contractAddress,
        contractAbi: contractAbi.format(ethers.utils.FormatTypes.json),
        durationSeconds,
        rpcUrl
      }
    });
    
    worker.on('message', (message) => {
      if (message.type === 'result') {
        totalRequests += message.data.totalRequests;
        successfulRequests += message.data.successfulRequests;
        failedRequests += message.data.failedRequests;
        totalLatency += message.data.totalLatency;
      }
    });
    
    workers.push(worker);
  }
  
  // Wait for test duration plus a small buffer
  await new Promise(resolve => setTimeout(resolve, (durationSeconds + 5) * 1000));
  
  // Stop workers and collect final metrics
  for (const worker of workers) {
    worker.terminate();
  }
  
  // Calculate and report results
  const averageLatency = successfulRequests > 0 ? totalLatency / successfulRequests : 0;
  const throughput = successfulRequests / durationSeconds;
  
  const results = {
    totalRequests,
    successfulRequests,
    failedRequests,
    errorRate: (failedRequests / totalRequests) * 100,
    averageLatency,
    throughput
  };
  
  console.log('Load Test Results:');
  console.log(`Total Requests: ${results.totalRequests}`);
  console.log(`Successful: ${results.successfulRequests}`);
  console.log(`Failed: ${results.failedRequests}`);
  console.log(`Error Rate: ${results.errorRate.toFixed(2)}%`);
  console.log(`Average Latency: ${results.averageLatency.toFixed(2)}ms`);
  console.log(`Throughput: ${results.throughput.toFixed(2)} req/sec`);
  
  // Save results to file
  fs.writeFileSync(
    `load-test-results-${Date.now()}.json`,
    JSON.stringify(results, null, 2)
  );
  
  return results;
}

// Worker implementation
if (!isMainThread) {
  const {
    wallets,
    requestsPerUser,
    contractAddress,
    contractAbi,
    durationSeconds,
    rpcUrl
  } = workerData;
  
  async function runWorker() {
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);
    const parsedAbi = JSON.parse(contractAbi);
    const contract = new ethers.Contract(contractAddress, parsedAbi, provider);
    
    // Metrics
    let totalRequests = 0;
    let successfulRequests = 0;
    let failedRequests = 0;
    let totalLatency = 0;
    
    const endTime = Date.now() + (durationSeconds * 1000);
    
    // Create wallet instances
    const walletInstances = wallets.map(pk => new ethers.Wallet(pk, provider));
    
    // Run load test until duration expires
    while (Date.now() < endTime && totalRequests < walletInstances.length * requestsPerUser) {
      for (const wallet of walletInstances) {
        if (Date.now() >= endTime) break;
        
        const startTime = Date.now();
        try {
          // Call contract method
          const connectedContract = contract.connect(wallet);
          const tx = await connectedContract.testMethod(Date.now());
          await tx.wait();
          
          successfulRequests++;
        } catch (error) {
          failedRequests++;
          console.error(`Error in worker: ${error.message}`);
        }
        
        totalRequests++;
        totalLatency += Date.now() - startTime;
      }
    }
    
    // Send results back to main thread
    parentPort.postMessage({
      type: 'result',
      data: {
        totalRequests,
        successfulRequests,
        failedRequests,
        totalLatency
      }
    });
  }
  
  runWorker().catch(error => {
    console.error(`Worker error: ${error}`);
    process.exit(1);
  });
}

module.exports = { runLoadTest };
```

### Stress Testing Transaction Processing

Pushing system limits to identify breaking points:

```javascript
describe("System Stress Tests", function() {
  let contract;
  let deployer;
  let stressers;
  
  before(async function() {
    // These tests can take time
    this.timeout(300000); // 5 minutes
    
    // Get signers
    [deployer, ...stressers] = await ethers.getSigners();
    
    // Deploy test contract
    const Contract = await ethers.getContractFactory("StressTestTarget");
    contract = await Contract.deploy();
  });
  
  it("handles high transaction volume", async function() {
    // Skip on actual networks
    if (network.name !== 'hardhat' && network.name !== 'localhost') {
      console.log('Skipping stress test on non-local network');
      this.skip();
    }
    
    const batchSize = 50; // Adjust based on your test environment
    const batches = 5;
    const results = [];
    
    for (let batch = 1; batch <= batches; batch++) {
      const numTransactions = batch * batchSize;
      console.log(`Running stress test with ${numTransactions} transactions...`);
      
      // Prepare transactions
      const promises = [];
      const startTime = Date.now();
      
      for (let i = 0; i < numTransactions; i++) {
        const wallet = stressers[i % stressers.length];
        promises.push(
          contract.connect(wallet).recordTransaction(i, { gasLimit: 100000 })
        );
      }
      
      // Execute all transactions
      const txs = await Promise.allSettled(promises);
      
      // Wait for confirmed receipts (for successful transactions)
      const receiptPromises = txs
        .filter(r => r.status === 'fulfilled')
        .map(r => r.value.wait());
      
      const receipts = await Promise.allSettled(receiptPromises);
      
      const endTime = Date.now();
      const executionTime = endTime - startTime;
      
      // Calculate results
      const successfulTxs = receipts.filter(r => r.status === 'fulfilled').length;
      const failedTxs = numTransactions - successfulTxs;
      const throughput = (successfulTxs / executionTime) * 1000; // tx per second
      
      results.push({
        numTransactions,
        executionTime,
        successfulTxs,
        failedTxs,
        throughput
      });
      
      console.log(`Results for ${numTransactions} transactions:`);
      console.log(`  Time: ${executionTime}ms`);
      console.log(`  Success: ${successfulTxs}/${numTransactions}`);
      console.log(`  Throughput: ${throughput.toFixed(2)} tx/s`);
    }
    
    // Find breaking point (if any)
    let breakingPoint = null;
    for (let i = 1; i < results.length; i++) {
      const prevSuccess = results[i-1].successfulTxs / results[i-1].numTransactions;
      const currSuccess = results[i].successfulTxs / results[i].numTransactions;
      
      // If success rate drops significantly, consider it a breaking point
      if (currSuccess < 0.8 * prevSuccess) {
        breakingPoint = results[i].numTransactions;
        break;
      }
    }
    
    if (breakingPoint) {
      console.log(`System breaking point identified at ~${breakingPoint} transactions`);
    } else {
      console.log('No clear breaking point identified in the tested range');
    }
  });
});
```

### Memory Usage Testing

Testing memory consumption in contracts:

```javascript
describe("Memory Usage Tests", function() {
  let factory;
  
  before(async function() {
    factory = await ethers.getContractFactory("MemoryUsageTest");
  });
  
  it("tests array growth impact on performance", async function() {
    // Deploy fresh contract for the test
    const contract = await factory.deploy();
    
    // Test different array sizes
    const arraySizes = [10, 50, 100, 500, 1000];
    const results = [];
    
    for (const size of arraySizes) {
      // Generate test data
      const data = Array.from({ length: size }, (_, i) => i);
      
      // Test storing the array
      const tx = await contract.storeArray(data);
      const receipt = await tx.wait();
      
      // Retrieve the array (read operation)
      const readTx = await contract.retrieveArray();
      
      results.push({
        size,
        gasUsed: receipt.gasUsed.toString()
      });
      
      console.log(`Array size ${size}: ${receipt.gasUsed.toString()} gas`);
    }
    
    // Check for exponential growth in gas costs
    // as array sizes increase linearly
    let isExponential = false;
    for (let i = 2; i < results.length; i++) {
      const ratio1 = ethers.BigNumber.from(results[i-1].gasUsed).div(ethers.BigNumber.from(results[i-2].gasUsed));
      const ratio2 = ethers.BigNumber.from(results[i].gasUsed).div(ethers.BigNumber.from(results[i-1].gasUsed));
      
      if (ratio2.gt(ratio1.mul(12).div(10))) {
        console.log(`Warning: Potentially exponential growth detected at array size ${results[i].size}`);
        isExponential = true;
        break;
      }
    }
    
    if (!isExponential) {
      console.log('Array size scaling appears to be linear or sub-exponential');
    }
  });
  
  it("tests mapping vs. array performance", async function() {
    // Deploy fresh contract for the test
    const contract = await factory.deploy();
    
    const dataPoints = 100;
    const ids = Array.from({ length: dataPoints }, (_, i) => i);
    const values = Array.from({ length: dataPoints }, (_, i) => i * 10);
    
    // Test array-based storage
    const arrayTx = await contract.storeInArray(ids, values);
    const arrayReceipt = await arrayTx.wait();
    
    // Test mapping-based storage
    const mappingTx = await contract.storeInMapping(ids, values);
    const mappingReceipt = await mappingTx.wait();
    
    console.log(`Array storage: ${arrayReceipt.gasUsed.toString()} gas`);
    console.log(`Mapping storage: ${mappingReceipt.gasUsed.toString()} gas`);
    
    // Compare retrieval gas costs
    let arrayRetrievalGas = ethers.BigNumber.from(0);
    let mappingRetrievalGas = ethers.BigNumber.from(0);
    
    // Sample a few random IDs for testing retrieval
    const sampleIds = [10, 25, 50, 75];
    
    for (const id of sampleIds) {
      // Estimate array retrieval gas
      arrayRetrievalGas = arrayRetrievalGas.add(
        await contract.estimateGas.getFromArray(id)
      );
      
      // Estimate mapping retrieval gas
      mappingRetrievalGas = mappingRetrievalGas.add(
        await contract.estimateGas.getFromMapping(id)
      );
    }
    
    const avgArrayRetrieval = arrayRetrievalGas.div(sampleIds.length);
    const avgMappingRetrieval = mappingRetrievalGas.div(sampleIds.length);
    
    console.log(`Average array retrieval: ${avgArrayRetrieval.toString()} gas`);
    console.log(`Average mapping retrieval: ${avgMappingRetrieval.toString()} gas`);
    
    // Mappings should typically be more efficient for retrieval
    expect(avgMappingRetrieval).to.be.lt(avgArrayRetrieval);
  });
});
```

## Performance Testing Tools

### Gas Profiler

Creating a gas profiling utility:

```javascript
// scripts/gas-profiler.js
const { ethers } = require('hardhat');
const fs = require('fs');

class GasProfiler {
  constructor(options = {}) {
    this.profiles = {};
    this.enabled = options.enabled !== false;
    this.outputPath = options.outputPath || './gas-profiles';
    this.currentTest = null;
    
    if (this.enabled && !fs.existsSync(this.outputPath)) {
      fs.mkdirSync(this.outputPath, { recursive: true });
    }
  }
  
  startTest(name) {
    if (!this.enabled) return;
    
    this.currentTest = name;
    this.profiles[name] = this.profiles[name] || {
      name,
      operations: {},
      transactions: []
    };
  }
  
  async measureGas(operationName, txPromise) {
    if (!this.enabled || !this.currentTest) return await txPromise;
    
    const startTime = Date.now();
    const tx = await txPromise;
    const receipt = await tx.wait();
    const endTime = Date.now();
    
    const profile = this.profiles[this.currentTest];
    const operation = profile.operations[operationName] || {
      name: operationName,
      calls: 0,
      totalGas: ethers.BigNumber.from(0),
      totalTime: 0
    };
    
    operation.calls++;
    operation.totalGas = operation.totalGas.add(receipt.gasUsed);
    operation.totalTime += (endTime - startTime);
    
    profile.operations[operationName] = operation;
    
    // Record individual transaction
    profile.transactions.push({
      operation: operationName,
      hash: tx.hash,
      gasUsed: receipt.gasUsed.toString(),
      time: endTime - startTime
    });
    
    return tx;
  }
  
  endTest() {
    if (!this.enabled || !this.currentTest) return;
    
    const profile = this.profiles[this.currentTest];
    
    // Calculate summary statistics
    profile.summary = {
      totalCalls: 0,
      totalGas: ethers.BigNumber.from(0),
      totalTime: 0,
      averageGasPerCall: 0,
      averageTimePerCall: 0,
      operations: Object.keys(profile.operations).length
    };
    
    // Process each operation for summary
    Object.values(profile.operations).forEach(op => {
      profile.summary.totalCalls += op.calls;
      profile.summary.totalGas = profile.summary.totalGas.add(op.totalGas);
      profile.summary.totalTime += op.totalTime;
      
      // Calculate averages for the operation
      op.averageGas = op.totalGas.div(op.calls).toString();
      op.averageTime = Math.round(op.totalTime / op.calls);
      op.totalGas = op.totalGas.toString();
    });
    
    // Calculate overall averages
    if (profile.summary.totalCalls > 0) {
      profile.summary.averageGasPerCall = profile.summary.totalGas.div(profile.summary.totalCalls).toString();
      profile.summary.averageTimePerCall = Math.round(profile.summary.totalTime / profile.summary.totalCalls);
    }
    
    profile.summary.totalGas = profile.summary.totalGas.toString();
    
    // Save profile to disk
    const filename = `${this.outputPath}/${this.currentTest.replace(/\s+/g, '-')}-${Date.now()}.json`;
    fs.writeFileSync(filename, JSON.stringify(profile, null, 2));
    
    this.currentTest = null;
  }
  
  generateReport() {
    if (!this.enabled) return;
    
    const allProfiles = Object.values(this.profiles);
    
    // Create summary report
    const report = {
      timestamp: new Date().toISOString(),
      profiles: allProfiles.length,
      summary: {
        totalTests: allProfiles.length,
        totalOperations: 0,
        totalCalls: 0,
        totalGas: ethers.BigNumber.from(0)
      },
      testSummaries: allProfiles.map(profile => ({
        name: profile.name,
        operations: Object.keys(profile.operations).length,
        calls: profile.summary?.totalCalls || 0,
        totalGas: profile.summary?.totalGas || '0',
        averageGas: profile.summary?.averageGasPerCall || '0'
      }))
    };
    
    // Aggregate stats across all profiles
    allProfiles.forEach(profile => {
      if (profile.summary) {
        report.summary.totalOperations += profile.summary.operations;
        report.summary.totalCalls += profile.summary.totalCalls;
        report.summary.totalGas = report.summary.totalGas.add(
          ethers.BigNumber.from(profile.summary.totalGas)
        );
      }
    });
    
    report.summary.totalGas = report.summary.totalGas.toString();
    
    // Save report
    const filename = `${this.outputPath}/gas-profile-summary-${Date.now()}.json`;
    fs.writeFileSync(filename, JSON.stringify(report, null, 2));
    
    return report;
  }
}

module.exports = { GasProfiler };
```

Example usage:

```javascript
const { GasProfiler } = require('../scripts/gas-profiler');

describe("Contract Performance with Profiler", function() {
  let contract;
  let profiler;
  
  before(async function() {
    profiler = new GasProfiler({ enabled: true });
    const Contract = await ethers.getContractFactory("YourContract");
    contract = await Contract.deploy();
  });
  
  it("profiles different operations", async function() {
    profiler.startTest("Basic Operations Profile");
    
    // Measure simple operation
    await profiler.measureGas("setValue", contract.setValue(100));
    
    // Measure complex operation
    await profiler.measureGas("complexOperation", contract.complexOperation([1, 2, 3]));
    
    // End test and save profile
    profiler.endTest();
  });
  
  after(function() {
    // Generate summary report
    const report = profiler.generateReport();
    console.log("Gas profiling complete:", report.summary);
  });
});
```

### Performance Regression Testing

Setting up tests to detect performance regressions:

```javascript
const fs = require('fs');
const path = require('path');
const { expect } = require('chai');
const { ethers } = require('hardhat');

describe("Performance Regression Tests", function() {
  let contract;
  const historyFile = path.join(__dirname, '../performance-history.json');
  let history = [];
  
  // Load previous performance history if available
  before(async function() {
    if (fs.existsSync(historyFile)) {
      history = JSON.parse(fs.readFileSync(historyFile, 'utf8'));
      console.log(`Loaded ${history.length} historical performance records`);
    }
    
    const Contract = await ethers.getContractFactory("PerformanceTest");
    contract = await Contract.deploy();
  });
  
  it("should maintain performance within acceptable thresholds", async function() {
    // Define test scenarios
    const scenarios = [
      { name: "simple_transfer", fn: () => contract.simpleTransfer(ethers.utils.parseEther("1.0")) },
      { name: "complex_calculation", fn: () => contract.complexCalculation(100, 200) },
      { name: "data_storage", fn: () => contract.storeData([1, 2, 3, 4, 5]) }
    ];
    
    const results = {};
    
    // Run each scenario and measure gas
    for (const scenario of scenarios) {
      const tx = await scenario.fn();
      const receipt = await tx.wait();
      
      // Record results
      results[scenario.name] = {
        gasUsed: receipt.gasUsed.toString()
      };
      
      console.log(`${scenario.name}: ${receipt.gasUsed.toString()} gas`);
    }
    
    // Compare with history if available
    if (history.length > 0) {
      const lastRecord = history[history.length - 1];
      
      for (const [name, result] of Object.entries(results)) {
        const currentGas = ethers.BigNumber.from(result.gasUsed);
        const historicalGas = ethers.BigNumber.from(lastRecord[name]?.gasUsed || '0');
        
        // Skip if no historical data
        if (historicalGas.eq(0)) continue;
        
        // Calculate change
        const diff = currentGas.sub(historicalGas);
        const percentChange = diff.mul(100).div(historicalGas);
        
        console.log(`${name}: ${percentChange.toString()}% change from previous`);
        
        // Assert no significant regression (>5% increase)
        if (percentChange.gt(5)) {
          console.warn(`⚠️ Performance regression detected in ${name}: ${percentChange.toString()}% increase in gas usage`);
        }
        
        // For critical paths, we can make this a hard failure
        if (name === "critical_operation" && percentChange.gt(10)) {
          expect(currentGas).to.be.lte(
            historicalGas.mul(110).div(100),
            `Critical operation gas increase exceeds 10%`
          );
        }
      }
    }
    
    // Save current results to history
    history.push({
      timestamp: new Date().toISOString(),
      commit: process.env.GIT_COMMIT || "unknown",
      ...results
    });
    
    // Keep history manageable (last 20 entries)
    if (history.length > 20) {
      history = history.slice(-20);
    }
    
    // Save updated history
    fs.writeFileSync(historyFile, JSON.stringify(history, null, 2));
  });
});
```

## Conclusion

Performance testing is a critical aspect of blockchain application development, ensuring that applications are not only functional but also efficient in terms of gas usage and transaction throughput. By systematically measuring and optimizing performance, developers can create applications that provide better user experiences while minimizing costs.

The techniques covered in this chapter—from gas optimization to load testing—provide a comprehensive approach to performance testing for ProzChain applications. By integrating these practices into your development workflow, you can identify performance issues early and ensure that your applications meet the demanding requirements of blockchain environments.

As blockchain technology continues to evolve and scale, the importance of performance testing will only increase. By establishing a strong foundation in performance testing methodologies, developers can build applications that remain efficient and cost-effective even as the underlying platforms change.

## Next Steps

- [Security Testing](./testing-framework-security-testing.md): Learn how to test for security vulnerabilities in your blockchain applications.
- [Property-Based Testing](./testing-framework-property-testing.md): Discover how to generate test cases and verify invariants.
- [Continuous Integration](./testing-framework-ci-index.md): Explore how to automate performance testing in CI pipelines.

