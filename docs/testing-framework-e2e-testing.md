# End-to-End Testing

## Overview

End-to-End (E2E) testing evaluates the entire blockchain system's workflow from start to finish. Unlike unit or integration tests that focus on individual components or limited interactions, E2E tests validate complete transaction flows and business processes across the entire ProzChain environment. This chapter explores techniques for setting up and running effective E2E tests for blockchain applications.

## Full System Testing Approach

### E2E Testing Philosophy

E2E testing for blockchain applications requires a different approach from traditional applications:

1. **Blockchain Finality**: Tests must account for block confirmation times and finality guarantees
2. **State Persistence**: Tests must verify system state changes across multiple blocks
3. **Cross-Component Validation**: Tests must verify that all system components (on-chain and off-chain) remain synchronized
4. **Transaction Lifecycle**: Tests must follow transactions through their complete lifecycle

### Setting Up E2E Test Structure

A well-structured E2E test follows this pattern:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");
const axios = require("axios");

describe("Complete Transaction Flow", function() {
  // Extend timeout for E2E tests
  this.timeout(60000); // 60 seconds
  
  let contracts = {};
  let accounts;
  let apiClient;
  
  before(async function() {
    // Deploy all required contracts
    accounts = await ethers.getSigners();
    
    // Deploy token contract
    const Token = await ethers.getContractFactory("ProzToken");
    contracts.token = await Token.deploy("Test Token", "TST");
    await contracts.token.deployed();
    
    // Deploy exchange contract
    const Exchange = await ethers.getContractFactory("ProzExchange");
    contracts.exchange = await Exchange.deploy(contracts.token.address);
    await contracts.exchange.deployed();
    
    // Set up API client for off-chain components
    apiClient = axios.create({
      baseURL: process.env.API_URL || "http://localhost:3000/api",
      timeout: 5000
    });
  });
  
  it("should process a token purchase end-to-end", async function() {
    // 1. Prepare accounts and balances
    const buyer = accounts[1];
    const initialEthBalance = await buyer.getBalance();
    
    // 2. Execute purchase transaction
    const purchaseAmount = ethers.utils.parseEther("1.0"); // 1 ETH
    const tx = await contracts.exchange.connect(buyer).buyTokens({
      value: purchaseAmount
    });
    
    // 3. Wait for transaction confirmation
    const receipt = await tx.wait(2); // Wait for 2 confirmations
    
    // 4. Verify on-chain state changes
    const finalEthBalance = await buyer.getBalance();
    const tokenBalance = await contracts.token.balanceOf(buyer.address);
    
    // Account for gas costs in ETH balance check
    const gasCost = receipt.gasUsed.mul(receipt.effectiveGasPrice);
    const expectedEthBalance = initialEthBalance.sub(purchaseAmount).sub(gasCost);
    
    expect(finalEthBalance).to.be.closeTo(expectedEthBalance, ethers.utils.parseEther("0.0001"));
    expect(tokenBalance).to.be.gt(0);
    
    // 5. Verify off-chain systems synchronized with blockchain
    const response = await apiClient.get(`/accounts/${buyer.address}`);
    expect(response.status).to.equal(200);
    expect(response.data.tokenBalance).to.equal(tokenBalance.toString());
    
    // 6. Verify event processing
    const events = await apiClient.get(`/events?address=${buyer.address}&type=Purchase`);
    expect(events.data.length).to.be.gte(1);
    expect(events.data[0].transactionHash).to.equal(receipt.transactionHash);
  });
});
```

### Testing Multiple User Interactions

E2E tests should verify scenarios involving multiple users:

```javascript
it("should process a complete token exchange between users", async function() {
  const seller = accounts[1];
  const buyer = accounts[2];
  const tokenAmount = ethers.utils.parseEther("100");
  
  // Prepare: Mint tokens to seller
  await contracts.token.mint(seller.address, tokenAmount);
  await contracts.token.connect(seller).approve(contracts.exchange.address, tokenAmount);
  
  // Step 1: Seller creates listing
  const listingPrice = ethers.utils.parseEther("1"); // 1 ETH per token
  await contracts.exchange.connect(seller).createListing(tokenAmount, listingPrice);
  
  // Step 2: Buyer purchases tokens
  const buyerInitialBalance = await contracts.token.balanceOf(buyer.address);
  await contracts.exchange.connect(buyer).fillOrder(0, { value: listingPrice });
  
  // Step 3: Verify token transfer
  const buyerFinalBalance = await contracts.token.balanceOf(buyer.address);
  expect(buyerFinalBalance.sub(buyerInitialBalance)).to.equal(tokenAmount);
  
  // Step 4: Verify seller received payment
  const listing = await contracts.exchange.getListing(0);
  expect(listing.filled).to.be.true;
});
```

## Test Network Setup

### Local Full Network Configuration

Setting up a complete test network environment:

```javascript
// test-helpers/localNetwork.js
const { ethers } = require("hardhat");
const { exec } = require("child_process");
const fs = require("fs");

async function setupLocalTestNetwork() {
  // Generate network configuration
  const config = {
    chainId: 31337,
    networkName: "proz-testnet",
    blockTime: 5, // seconds
    initialAccounts: 10,
    initialBalance: "10000000000000000000000", // 10,000 ETH
  };
  
  // Start local network
  const node = exec("npx hardhat node --config test-hardhat-config.js");
  
  // Wait for node to start
  await new Promise(resolve => setTimeout(resolve, 5000));
  
  // Deploy core contracts
  const coreContracts = await deployCoreContracts();
  
  // Start auxiliary services
  const services = await startAuxiliaryServices(coreContracts);
  
  return {
    provider: ethers.provider,
    coreContracts,
    services,
    accounts: await ethers.getSigners(),
    shutdown: async () => {
      // Cleanup function
      await stopServices(services);
      node.kill();
    }
  };
}

async function deployCoreContracts() {
  // Deploy core contracts needed for all tests
  const Token = await ethers.getContractFactory("ProzToken");
  const token = await Token.deploy("ProzChain", "PROZ");
  
  const Registry = await ethers.getContractFactory("ContractRegistry");
  const registry = await Registry.deploy();
  
  // Register token in registry
  await registry.register("token", token.address);
  
  return { token, registry };
}

async function startAuxiliaryServices(contracts) {
  // Start indexer, API server, etc.
  // This is a placeholder - actual implementation would depend on your services
  
  return {
    indexer: { url: "http://localhost:3000" },
    api: { url: "http://localhost:4000" }
  };
}

async function stopServices(services) {
  // Stop all auxiliary services
  // This is a placeholder
  console.log("Stopping auxiliary services");
}

module.exports = { setupLocalTestNetwork };
```

### Docker-Based Test Environment

Using Docker for reproducible test environments:

```javascript
// test-helpers/dockerEnvironment.js
const { exec } = require("child_process");
const axios = require("axios");
const { ethers } = require("ethers");
const path = require("path");

async function setupDockerTestNetwork() {
  // Start docker environment using docker-compose
  console.log("Starting Docker test environment");
  
  const composeFile = path.join(__dirname, "../docker/e2e-test-compose.yml");
  
  try {
    // Pull latest images
    await execCommand(`docker-compose -f ${composeFile} pull`);
    
    // Start services
    await execCommand(`docker-compose -f ${composeFile} up -d`);
    
    // Wait for services to be ready
    await waitForServices();
    
    // Connect to blockchain node
    const provider = new ethers.providers.JsonRpcProvider("http://localhost:8545");
    
    // Load private keys provided by the docker environment
    const privateKeys = JSON.parse(process.env.TEST_PRIVATE_KEYS || '[]');
    const signers = privateKeys.map(key => new ethers.Wallet(key, provider));
    
    return {
      provider,
      signers,
      shutdown: async () => {
        // Shut down docker environment
        await execCommand(`docker-compose -f ${composeFile} down -v`);
      }
    };
  } catch (error) {
    console.error("Failed to start Docker environment:", error);
    throw error;
  }
}

async function execCommand(command) {
  return new Promise((resolve, reject) => {
    exec(command, (error, stdout, stderr) => {
      if (error) {
        console.error(`Command failed: ${error}`);
        reject(error);
        return;
      }
      resolve(stdout.trim());
    });
  });
}

async function waitForServices() {
  // Wait for blockchain node to be ready
  await waitForService("http://localhost:8545", async () => {
    try {
      const provider = new ethers.providers.JsonRpcProvider("http://localhost:8545");
      await provider.getBlockNumber();
      return true;
    } catch (error) {
      return false;
    }
  });
  
  // Wait for API service to be ready
  await waitForService("http://localhost:3000/api/health", async () => {
    try {
      const response = await axios.get("http://localhost:3000/api/health");
      return response.status === 200;
    } catch (error) {
      return false;
    }
  });
}

async function waitForService(name, checkFn, timeoutMs = 60000, intervalMs = 1000) {
  const startTime = Date.now();
  while (Date.now() - startTime < timeoutMs) {
    if (await checkFn()) {
      console.log(`Service ${name} is ready`);
      return;
    }
    await new Promise(resolve => setTimeout(resolve, intervalMs));
  }
  throw new Error(`Timeout waiting for service ${name}`);
}

module.exports = { setupDockerTestNetwork };
```

### Testnet Integration

Setting up E2E tests on public testnets:

```javascript
// test-helpers/testnetSetup.js
const { ethers } = require("ethers");
const fs = require("fs");
const path = require("path");

async function setupTestnetEnvironment() {
  const networkName = process.env.TESTNET || "prozchain-testnet";
  
  // Get network configuration
  const config = getNetworkConfig(networkName);
  
  // Create provider
  const provider = new ethers.providers.JsonRpcProvider(config.rpcUrl);
  
  // Get or create wallet
  const wallet = getTestWallet(provider);
  
  // Ensure wallet has funds
  await ensureWalletFunded(wallet, config);
  
  // Load existing contract deployments
  const deployments = loadDeployments(networkName);
  
  // Connect to contracts
  const contracts = await connectToContracts(deployments, wallet);
  
  return {
    provider,
    wallet,
    contracts,
    networkName,
    networkConfig: config
  };
}

function getNetworkConfig(networkName) {
  const networks = {
    "prozchain-testnet": {
      rpcUrl: process.env.PROZCHAIN_TESTNET_RPC || "https://testnet-rpc.prozchain.com",
      chainId: 24601,
      faucetUrl: "https://faucet.prozchain.com/request",
      blockExplorer: "https://testnet.prozscan.io",
      minFunds: ethers.utils.parseEther("0.1")
    },
    "sepolia": {
      rpcUrl: process.env.SEPOLIA_RPC || "https://sepolia.infura.io/v3/your-api-key",
      chainId: 11155111,
      faucetUrl: "https://sepoliafaucet.com",
      blockExplorer: "https://sepolia.etherscan.io",
      minFunds: ethers.utils.parseEther("0.05")
    }
  };
  
  if (!networks[networkName]) {
    throw new Error(`Network ${networkName} is not supported for E2E tests`);
  }
  
  return networks[networkName];
}

function getTestWallet(provider) {
  // Use private key from environment or create deterministic wallet
  if (process.env.TEST_PRIVATE_KEY) {
    return new ethers.Wallet(process.env.TEST_PRIVATE_KEY, provider);
  }
  
  // Create deterministic wallet for testing (DO NOT USE FOR REAL FUNDS)
  const mnemonic = "test test test test test test test test test test test junk";
  return ethers.Wallet.fromMnemonic(mnemonic).connect(provider);
}

async function ensureWalletFunded(wallet, config) {
  // Check wallet balance
  const balance = await wallet.getBalance();
  
  if (balance.lt(config.minFunds)) {
    console.log(`Wallet ${wallet.address} has insufficient funds: ${ethers.utils.formatEther(balance)} ETH`);
    
    // Request funds from faucet if not a production network
    if (config.faucetUrl) {
      console.log(`Requesting funds from ${config.faucetUrl}`);
      // Implementation would depend on the specific faucet API
      // This is just a placeholder
      
      // Wait for funds
      await waitForFunds(wallet, config.minFunds);
    } else {
      throw new Error(`Insufficient funds for testing and no faucet available`);
    }
  }
}

async function waitForFunds(wallet, minAmount) {
  const maxWaitTime = 60_000; // 60 seconds
  const interval = 5_000; // Check every 5 seconds
  const startTime = Date.now();
  
  while (Date.now() - startTime < maxWaitTime) {
    const balance = await wallet.getBalance();
    if (balance.gte(minAmount)) {
      console.log(`Received funds. Balance: ${ethers.utils.formatEther(balance)} ETH`);
      return;
    }
    await new Promise(resolve => setTimeout(resolve, interval));
  }
  
  throw new Error("Timed out waiting for funds");
}

function loadDeployments(networkName) {
  const deploymentsPath = path.join(__dirname, "../deployments", networkName, "addresses.json");
  
  if (!fs.existsSync(deploymentsPath)) {
    throw new Error(`No deployments found for ${networkName}`);
  }
  
  return JSON.parse(fs.readFileSync(deploymentsPath, "utf8"));
}

async function connectToContracts(deployments, wallet) {
  const contracts = {};
  const artifactsDir = path.join(__dirname, "../artifacts/contracts");
  
  for (const [name, address] of Object.entries(deployments)) {
    // Find artifact file
    const artifactFiles = fs.readdirSync(artifactsDir, { recursive: true })
      .filter(file => file.endsWith(`${name}.json`));
    
    if (artifactFiles.length === 0) {
      console.warn(`No artifact found for contract ${name}`);
      continue;
    }
    
    const artifactPath = path.join(artifactsDir, artifactFiles[0]);
    const artifact = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
    
    // Connect to contract
    contracts[name] = new ethers.Contract(address, artifact.abi, wallet);
    console.log(`Connected to ${name} at ${address}`);
  }
  
  return contracts;
}

module.exports = { setupTestnetEnvironment };
```

## Transaction Flow Testing

### Complete Transaction Lifecycle Test

Testing the entire lifecycle of a transaction:

```javascript
describe("Transaction Lifecycle", function() {
  this.timeout(120000); // 2 minutes
  
  let network;
  let contracts;
  let wallet;
  
  before(async function() {
    // Setup test environment
    const testEnv = await setupTestnetEnvironment();
    network = testEnv.networkName;
    contracts = testEnv.contracts;
    wallet = testEnv.wallet;
  });
  
  it("should follow a transaction through its complete lifecycle", async function() {
    // 1. Create transaction
    console.log("Creating transaction...");
    const recipient = ethers.Wallet.createRandom().address;
    const txAmount = ethers.utils.parseEther("0.01");
    
    // 2. Submit to mempool
    console.log("Submitting transaction...");
    const tx = await wallet.sendTransaction({
      to: recipient,
      value: txAmount
    });
    console.log(`Transaction hash: ${tx.hash}`);
    
    // 3. Track mempool acceptance
    console.log("Waiting for transaction to be included in mempool...");
    let isPending = true;
    while (isPending) {
      const txnStatus = await wallet.provider.getTransaction(tx.hash);
      isPending = txnStatus && txnStatus.blockNumber === null;
      if (!isPending) break;
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    // 4. Track block inclusion
    console.log("Waiting for transaction to be mined...");
    const receipt = await tx.wait(1);
    console.log(`Transaction mined in block ${receipt.blockNumber}`);
    
    // 5. Verify transaction details
    console.log("Verifying transaction details...");
    const minedTx = await wallet.provider.getTransaction(tx.hash);
    expect(minedTx.from).to.equal(wallet.address);
    expect(minedTx.to).to.equal(recipient);
    expect(minedTx.value.toString()).to.equal(txAmount.toString());
    
    // 6. Wait for finality (depends on network)
    const finalityBlocks = getFinality(network);
    console.log(`Waiting for ${finalityBlocks} confirmations for finality...`);
    await tx.wait(finalityBlocks);
    
    // 7. Verify recipient state change
    const recipientBalance = await wallet.provider.getBalance(recipient);
    expect(recipientBalance.toString()).to.equal(txAmount.toString());
    
    // 8. Verify transaction in indexer/explorer (if available)
    if (process.env.EXPLORER_API) {
      const response = await axios.get(
        `${process.env.EXPLORER_API}/api/v1/transactions/${tx.hash}`
      );
      expect(response.data.status).to.equal("confirmed");
    }
  });
});

function getFinality(network) {
  const finality = {
    "prozchain-testnet": 5,
    "sepolia": 12,
    "hardhat": 1
  };
  
  return finality[network] || 1;
}
```

### Event Propagation Testing

Testing the propagation and processing of contract events:

```javascript
describe("Event Propagation Testing", function() {
  let eventProcessor;
  let token;
  let accounts;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy token contract
    const Token = await ethers.getContractFactory("ProzToken");
    token = await Token.deploy("Event Test Token", "EVT");
    await token.deployed();
    
    // Initialize event processor
    eventProcessor = new EventProcessorClient(process.env.EVENT_PROCESSOR_URL);
    
    // Register token for event monitoring
    await eventProcessor.registerContract(token.address, "Token");
  });
  
  it("should propagate transfer events to external systems", async function() {
    // 1. Prepare for event monitoring
    const recipient = accounts[1].address;
    const amount = ethers.utils.parseEther("100");
    
    // 2. Setup event listener
    const eventPromise = waitForEvent(eventProcessor, "Transfer", {
      contract: token.address,
      from: accounts[0].address,
      to: recipient
    });
    
    // 3. Execute transaction that emits event
    const tx = await token.transfer(recipient, amount);
    const receipt = await tx.wait();
    
    // Verify event was emitted on-chain
    const transferEvent = receipt.events?.find(e => e.event === "Transfer");
    expect(transferEvent).to.exist;
    expect(transferEvent.args.from).to.equal(accounts[0].address);
    expect(transferEvent.args.to).to.equal(recipient);
    expect(transferEvent.args.value).to.equal(amount);
    
    // 4. Wait for event to propagate to external system
    const processedEvent = await eventPromise;
    
    // 5. Verify event was processed correctly
    expect(processedEvent.transactionHash).to.equal(tx.hash);
    expect(processedEvent.data.from).to.equal(accounts[0].address.toLowerCase());
    expect(processedEvent.data.to).to.equal(recipient.toLowerCase());
    expect(processedEvent.data.value).to.equal(amount.toString());
  });
});

async function waitForEvent(eventProcessor, eventName, filters, timeoutMs = 30000) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      reject(new Error(`Timeout waiting for ${eventName} event`));
    }, timeoutMs);
    
    const checkInterval = setInterval(async () => {
      try {
        const events = await eventProcessor.getEvents({
          eventName,
          ...filters
        });
        
        if (events.length > 0) {
          clearTimeout(timeout);
          clearInterval(checkInterval);
          resolve(events[0]);
        }
      } catch (error) {
        console.error("Error checking for events:", error);
      }
    }, 1000); // Check every second
  });
}
```

## Block Production and Validation Testing

### Testing Block Production

Verifying correct block production and validation:

```javascript
describe("Block Production and Validation", function() {
  this.timeout(300000); // 5 minutes
  
  let provider;
  let miners;
  let testNode;
  
  before(async function() {
    // This test requires a specialized test network with controlled miners
    // Skip if not in appropriate testing environment
    if (!process.env.CONSENSUS_TEST_NETWORK) {
      console.log("Skipping block production tests - requires consensus test network");
      this.skip();
      return;
    }
    
    // Connect to specialized test network
    provider = new ethers.providers.JsonRpcProvider(process.env.CONSENSUS_TEST_RPC);
    
    // Connect to test miners (requires special test setup)
    miners = await connectToTestMiners();
    
    // Start monitoring node
    testNode = await startTestNode();
  });
  
  after(async function() {
    // Clean up test resources
    if (testNode) await testNode.shutdown();
  });
  
  it("should produce blocks at expected intervals", async function() {
    // Monitor block production
    const blockTimes = [];
    const blocksToMonitor = 5;
    
    for (let i = 0; i < blocksToMonitor; i++) {
      const newBlockPromise = waitForNextBlock(provider);
      const block = await newBlockPromise;
      blockTimes.push(block.timestamp);
      console.log(`Block ${block.number} produced at timestamp ${block.timestamp}`);
    }
    
    // Calculate average block time
    let totalTime = 0;
    for (let i = 1; i < blockTimes.length; i++) {
      totalTime += blockTimes[i] - blockTimes[i-1];
    }
    const averageBlockTime = totalTime / (blockTimes.length - 1);
    
    // Verify block time is close to expected
    const expectedBlockTime = getExpectedBlockTime();
    console.log(`Average block time: ${averageBlockTime} seconds`);
    console.log(`Expected block time: ${expectedBlockTime} seconds`);
    
    // Allow 20% variance in block time
    expect(averageBlockTime).to.be.closeTo(expectedBlockTime, expectedBlockTime * 0.2);
  });
  
  it("should validate blocks according to consensus rules", async function() {
    // This test creates an invalid block and verifies it's rejected
    
    // Get current validator set
    const validators = await testNode.getValidators();
    expect(validators.length).to.be.gt(0);
    
    // Create intentionally invalid block
    const invalidBlock = await miners[0].createInvalidBlock({
      // Set invalid parameters based on consensus rules
      timestamp: Math.floor(Date.now() / 1000) + 1000, // Future timestamp
      extraData: "0xInvalidExtraData"
    });
    
    // Submit invalid block
    try {
      await miners[0].submitBlock(invalidBlock);
      expect.fail("Invalid block was accepted");
    } catch (error) {
      // Verify block was rejected with appropriate error
      expect(error.message).to.include("invalid block");
    }
    
    // Verify chain state wasn't affected
    const latestBlock = await provider.getBlock("latest");
    expect(latestBlock.hash).to.not.equal(invalidBlock.hash);
  });
});

async function waitForNextBlock(provider) {
  return new Promise((resolve) => {
    provider.once("block", (blockNumber) => {
      provider.getBlock(blockNumber).then(resolve);
    });
  });
}

function getExpectedBlockTime() {
  const network = process.env.CONSENSUS_TEST_NETWORK;
  const blockTimes = {
    "prozchain-testnet": 3, // 3 seconds
    "prozchain-devnet": 1    // 1 second
  };
  return blockTimes[network] || 5;
}
```

### Chain Reorganization Testing

Testing blockchain behavior during network partitioning and reorganizations:

```javascript
describe("Chain Reorganization Testing", function() {
  this.timeout(600000); // 10 minutes
  
  let testNetwork;
  let primaryNode;
  let secondaryNode;
  let fork;
  
  before(async function() {
    // This test requires special test network that can be partitioned
    if (!process.env.REORG_TEST_ENABLED) {
      console.log("Skipping reorganization tests - required environment not available");
      this.skip();
      return;
    }
    
    // Set up isolated test network
    testNetwork = await setupIsolatedNetwork();
    
    // Get node connections
    primaryNode = testNetwork.getPrimaryNode();
    secondaryNode = testNetwork.getSecondaryNode();
  });
  
  after(async function() {
    // Clean up test resources
    if (testNetwork) await testNetwork.shutdown();
  });
  
  it("should handle chain reorganization correctly", async function() {
    // 1. Create network partition
    await testNetwork.createPartition();
    
    // 2. Generate transactions on primary partition
    const primaryWallet = await primaryNode.createFundedWallet();
    const primaryTx = await primaryWallet.sendTransaction({
      to: ethers.Wallet.createRandom().address,
      value: ethers.utils.parseEther("1.0")
    });
    const primaryReceipt = await primaryTx.wait();
    console.log(`Primary transaction included in block ${primaryReceipt.blockNumber}`);
    
    // 3. Generate longer chain on secondary partition
    const secondaryWallet = await secondaryNode.createFundedWallet();
    
    // Create multiple blocks to make secondary chain longer
    const secondaryTxs = [];
    for (let i = 0; i < 5; i++) {
      const tx = await secondaryWallet.sendTransaction({
        to: ethers.Wallet.createRandom().address,
        value: ethers.utils.parseEther("0.1")
      });
      secondaryTxs.push(tx);
      await tx.wait();
    }
    
    console.log(`Generated ${secondaryTxs.length} blocks on secondary partition`);
    
    // 4. Heal network partition
    await testNetwork.healPartition();
    console.log("Network partition healed");
    
    // 5. Wait for network to converge
    await testNetwork.waitForConvergence();
    console.log("Network converged");
    
    // 6. Verify chain reorganization
    const primaryTxConfirmation = await primaryNode.provider.getTransactionReceipt(primaryTx.hash);
    
    if (primaryTxConfirmation !== null) {
      console.log("Primary transaction still included - checking if block changed");
      expect(primaryTxConfirmation.blockNumber).to.not.equal(primaryReceipt.blockNumber);
    } else {
      console.log("Primary transaction was removed from the chain");
    }
    
    // 7. Verify secondary chain became canonical
    for (const tx of secondaryTxs) {
      const receipt = await primaryNode.provider.getTransactionReceipt(tx.hash);
      expect(receipt).to.not.be.null;
      console.log(`Secondary transaction ${tx.hash} included in block ${receipt.blockNumber}`);
    }
  });
});
```

## Data Consistency Testing

### Blockchain State Verification

Testing data consistency across blockchain and off-chain systems:

```javascript
describe("Data Consistency Testing", function() {
  let contracts;
  let accounts;
  let indexer;
  let api;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy test contracts
    const Token = await ethers.getContractFactory("ProzToken");
    contracts = {};
    contracts.token = await Token.deploy("Data Token", "DATA");
    await contracts.token.deployed();
    
    // Set up connections to indexer and API services
    indexer = new IndexerClient(process.env.INDEXER_URL || "http://localhost:3000");
    api = new ApiClient(process.env.API_URL || "http://localhost:4000");
  });
  
  it("should maintain consistency between blockchain and database", async function() {
    // 1. Create state-changing transaction
    const recipient = accounts[1].address;
    const amount = ethers.utils.parseEther("100");
    
    const tx = await contracts.token.transfer(recipient, amount);
    const receipt = await tx.wait(2); // Wait for confirmations
    
    console.log(`State change transaction confirmed in block ${receipt.blockNumber}`);
    
    // 2. Wait for indexer to process the block
    await waitForIndexer(indexer, receipt.blockNumber);
    console.log(`Indexer processed block ${receipt.blockNumber}`);
    
    // 3. Verify on-chain state
    const onChainBalance = await contracts.token.balanceOf(recipient);
    expect(onChainBalance).to.equal(amount);
    
    // 4. Verify indexed data
    const indexedTransfer = await indexer.getTransferByTxHash(tx.hash);
    expect(indexedTransfer).to.not.be.null;
    expect(indexedTransfer.from).to.equal(accounts[0].address.toLowerCase());
    expect(indexedTransfer.to).to.equal(recipient.toLowerCase());
    expect(indexedTransfer.value).to.equal(amount.toString());
    
    // 5. Verify API data
    const apiBalance = await api.getTokenBalance(recipient, contracts.token.address);
    expect(apiBalance).to.equal(amount.toString());
  });
  
  it("should handle complex state transitions consistently", async function() {
    // Create a more complex scenario with multiple interacting transactions
    
    // 1. Setup initial state
    const user1 = accounts[2];
    const user2 = accounts[3];
    await contracts.token.transfer(user1.address, ethers.utils.parseEther("1000"));
    
    // 2. Execute batch of transactions
    const batchSize = 5;
    const txHashes = [];
    const expectedBalances = {};
    
    for (let i = 0; i < batchSize; i++) {
      // Alternate between transactions from user1 and user2
      const sender = i % 2 === 0 ? user1 : user2;
      const recipient = i % 2 === 0 ? user2 : user1;
      
      const amount = ethers.utils.parseEther((i + 1).toString());
      
      // Update expected balances
      expectedBalances[sender.address] = (expectedBalances[sender.address] || ethers.BigNumber.from(0)).sub(amount);
      expectedBalances[recipient.address] = (expectedBalances[recipient.address] || ethers.BigNumber.from(0)).add(amount);
      
      // Send transaction
      const tx = await contracts.token.connect(sender).transfer(recipient.address, amount);
      const receipt = await tx.wait();
      txHashes.push(tx.hash);
    }
    
    // 3. Wait for indexer to catch up
    const latestBlock = await ethers.provider.getBlockNumber();
    await waitForIndexer(indexer, latestBlock);
    
    // 4. Verify final state
    for (const [address, expectedBalance] of Object.entries(expectedBalances)) {
      // Get initial balance
      const initialBalance = await contracts.token.balanceOf(accounts[0].address);
      
      // Add expected change
      const finalBalanceExpected = initialBalance.add(expectedBalance);
      
      // Check on-chain balance
      const onChainBalance = await contracts.token.balanceOf(address);
      expect(onChainBalance).to.equal(finalBalanceExpected);
      
      // Check indexed balance
      const indexedBalance = await indexer.getTokenBalance(address, contracts.token.address);
      expect(indexedBalance).to.equal(onChainBalance.toString());
      
      // Check API balance
      const apiBalance = await api.getTokenBalance(address, contracts.token.address);
      expect(apiBalance).to.equal(onChainBalance.toString());
    }
  });
});

async function waitForIndexer(indexer, targetBlockNumber, timeoutMs = 60000) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeoutMs) {
    const indexerBlock = await indexer.getLatestIndexedBlock();
    
    if (indexerBlock >= targetBlockNumber) {
      return true;
    }
    
    console.log(`Waiting for indexer: current=${indexerBlock}, target=${targetBlockNumber}`);
    await new Promise(resolve => setTimeout(resolve, 2000));
  }
  
  throw new Error(`Indexer failed to reach block ${targetBlockNumber} within timeout`);
}
```

## Conclusion

End-to-End testing is essential for blockchain applications to verify that all components work together correctly in real-world scenarios. By testing complete transaction flows, block production, chain reorganizations, and data consistency, developers can gain confidence that the system will behave as expected in production.

Key aspects of effective E2E testing for blockchain applications include:

1. Setting up comprehensive test environments that simulate all aspects of the production system
2. Testing complete transaction lifecycles from creation to finality
3. Verifying data consistency between on-chain and off-chain components
4. Testing resilience to network events like partitioning and reorganizations
5. Automating tests for continuous integration and regression detection

While E2E tests are more complex and slower to run than unit or integration tests, they provide invaluable assurance that the complete system functions correctly as a whole. By following the practices outlined in this chapter, you can create E2E tests that detect issues that might be missed by narrower testing approaches.
