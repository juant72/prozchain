# Mock Systems

## Overview

Mock systems are essential for blockchain testing, allowing developers to simulate external dependencies, blockchain components, and network conditions in a controlled environment. This chapter explores various mocking techniques specifically tailored for ProzChain applications, enabling comprehensive testing without relying on actual external systems or networks.

Using mocks effectively allows developers to test functionality in isolation, simulate edge cases that would be difficult to reproduce with real systems, and create fast, deterministic tests that aren't affected by the unpredictability of real blockchain networks.

## Mocking External Dependencies

### API Mocking

Simulating external API responses for testing:

```javascript
const nock = require('nock');
const { expect } = require('chai');
const { ethers } = require('hardhat');
const ExternalDataConsumer = require('../src/ExternalDataConsumer');

describe("External API Integration", function() {
  let dataConsumer;
  let mockApi;
  
  beforeEach(async function() {
    // Create a new instance of the consumer
    dataConsumer = new ExternalDataConsumer('https://api.example.com');
    
    // Set up API mock
    mockApi = nock('https://api.example.com');
  });
  
  afterEach(function() {
    // Clean up nock mocks
    nock.cleanAll();
  });
  
  it("fetches and processes price data correctly", async function() {
    // Setup mock response
    mockApi.get('/prices/ETH')
      .reply(200, {
        symbol: 'ETH',
        price: '2000.50',
        timestamp: Date.now()
      });
    
    // Call the method that uses the external API
    const result = await dataConsumer.getLatestPrice('ETH');
    
    // Verify the result
    expect(result).to.have.property('symbol', 'ETH');
    expect(result).to.have.property('price', '2000.50');
    expect(result).to.have.property('timestamp');
  });
  
  it("handles API errors gracefully", async function() {
    // Setup mock API error response
    mockApi.get('/prices/UNKNOWN')
      .reply(404, {
        error: 'Symbol not found'
      });
    
    // Verify error handling
    try {
      await dataConsumer.getLatestPrice('UNKNOWN');
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error.message).to.include('Symbol not found');
    }
  });
  
  it("retries on temporary failures", async function() {
    // First request fails with 503
    mockApi.get('/prices/ETH')
      .reply(503, { error: 'Service unavailable' });
    
    // Second request succeeds
    mockApi.get('/prices/ETH')
      .reply(200, { 
        symbol: 'ETH', 
        price: '2000.50',
        timestamp: Date.now()
      });
    
    // Call method that should retry
    const result = await dataConsumer.getPriceWithRetry('ETH');
    
    // Verify result
    expect(result).to.have.property('symbol', 'ETH');
    expect(result).to.have.property('price', '2000.50');
  });
});
```

### IPFS Mocking

Simulating IPFS interactions:

```javascript
const { expect } = require('chai');
const sinon = require('sinon');
const IPFSService = require('../src/IPFSService');

describe("IPFS Service", function() {
  let ipfsService;
  let mockIpfsClient;
  
  beforeEach(function() {
    // Create mock IPFS client with stubbed methods
    mockIpfsClient = {
      add: sinon.stub(),
      cat: sinon.stub(),
      pin: {
        add: sinon.stub()
      }
    };
    
    // Create service instance with mocked client
    ipfsService = new IPFSService(mockIpfsClient);
  });
  
  it("uploads data to IPFS", async function() {
    // Setup mock response
    const mockCid = 'QmXjkFQjnD8i8ntmwehoAHBfJEApETx8YTdefDEUCPsT1g';
    mockIpfsClient.add.resolves({
      path: mockCid,
      size: 123
    });
    
    // Test upload functionality
    const data = { name: 'Test NFT', description: 'Test Description' };
    const result = await ipfsService.uploadJson(data);
    
    // Verify correct CID is returned
    expect(result).to.equal(mockCid);
    
    // Verify IPFS client was called correctly
    expect(mockIpfsClient.add.calledOnce).to.be.true;
    const callArgs = mockIpfsClient.add.firstCall.args[0];
    expect(JSON.parse(callArgs)).to.deep.equal(data);
  });
  
  it("retrieves data from IPFS", async function() {
    // Mock data to be returned
    const mockData = JSON.stringify({ name: 'Test NFT', description: 'Test Description' });
    
    // Setup mock response - if using a stream mock
    const mockStream = {
      next: sinon.stub()
    };
    
    // First call returns the data chunk, second indicates end of stream
    mockStream.next.onFirstCall().resolves({
      done: false,
      value: Buffer.from(mockData)
    });
    mockStream.next.onSecondCall().resolves({
      done: true
    });
    
    mockIpfsClient.cat.returns(mockStream);
    
    // Test retrieval functionality
    const cid = 'QmXjkFQjnD8i8ntmwehoAHBfJEApETx8YTdefDEUCPsT1g';
    const result = await ipfsService.getJson(cid);
    
    // Verify data was retrieved correctly
    expect(result).to.deep.equal(JSON.parse(mockData));
    
    // Verify IPFS client was called correctly
    expect(mockIpfsClient.cat.calledOnce).to.be.true;
    expect(mockIpfsClient.cat.firstCall.args[0]).to.equal(cid);
  });
  
  it("handles IPFS errors gracefully", async function() {
    // Setup mock to throw error
    mockIpfsClient.cat.rejects(new Error('IPFS error'));
    
    // Test error handling
    try {
      await ipfsService.getJson('invalid-cid');
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error.message).to.include('IPFS error');
    }
  });
});
```

### Database Mocking

Creating mock database interactions:

```javascript
const { expect } = require('chai');
const sinon = require('sinon');
const { MongoClient } = require('mongodb');
const BlockchainEventProcessor = require('../src/BlockchainEventProcessor');

describe("Database Integration", function() {
  // MongoDB mocks
  let mockCollection;
  let mockDb;
  let mockClient;
  let processor;
  
  beforeEach(function() {
    // Create mock collection with stubbed methods
    mockCollection = {
      insertOne: sinon.stub().resolves({ insertedId: 'mock-id' }),
      findOne: sinon.stub(),
      updateOne: sinon.stub().resolves({ modifiedCount: 1 })
    };
    
    // Create mock database and client
    mockDb = {
      collection: sinon.stub().returns(mockCollection)
    };
    
    mockClient = {
      db: sinon.stub().returns(mockDb),
      connect: sinon.stub().resolves(),
      close: sinon.stub().resolves()
    };
    
    // Stub the MongoClient constructor
    sinon.stub(MongoClient, 'connect').resolves(mockClient);
    
    // Create processor with mocked dependencies
    processor = new BlockchainEventProcessor('mongodb://localhost:27017');
  });
  
  afterEach(function() {
    // Restore all stubs
    sinon.restore();
  });
  
  it("stores blockchain events in database", async function() {
    // Mock blockchain event
    const event = {
      transactionHash: '0x123',
      blockNumber: 100,
      args: {
        tokenId: '1',
        from: '0xabc',
        to: '0xdef',
        value: '1000000000000000000'
      }
    };
    
    // Process the event
    await processor.processEvent(event);
    
    // Verify database interaction
    expect(mockDb.collection.calledWith('events')).to.be.true;
    expect(mockCollection.insertOne.calledOnce).to.be.true;
    
    // Verify correct data was stored
    const storedData = mockCollection.insertOne.firstCall.args[0];
    expect(storedData).to.have.property('transactionHash', '0x123');
    expect(storedData).to.have.property('blockNumber', 100);
    expect(storedData).to.have.property('tokenId', '1');
    expect(storedData).to.have.property('from', '0xabc');
    expect(storedData).to.have.property('to', '0xdef');
  });
  
  it("retrieves event details by ID", async function() {
    // Setup mock response
    const mockEvent = {
      _id: 'event-id',
      transactionHash: '0x123',
      blockNumber: 100,
      tokenId: '1',
      processed: true
    };
    mockCollection.findOne.resolves(mockEvent);
    
    // Retrieve event
    const result = await processor.getEventById('event-id');
    
    // Verify correct event was returned
    expect(result).to.deep.equal(mockEvent);
    expect(mockCollection.findOne.calledWith({ _id: 'event-id' })).to.be.true;
  });
  
  it("updates event processing status", async function() {
    // Update event status
    await processor.markEventProcessed('event-id');
    
    // Verify database update
    expect(mockCollection.updateOne.calledOnce).to.be.true;
    
    // Check update parameters
    const updateQuery = mockCollection.updateOne.firstCall.args[0];
    const updateValues = mockCollection.updateOne.firstCall.args[1];
    
    expect(updateQuery).to.deep.equal({ _id: 'event-id' });
    expect(updateValues).to.have.property('$set');
    expect(updateValues.$set).to.have.property('processed', true);
    expect(updateValues.$set).to.have.property('processedAt');
  });
});
```

### Storage Mocking

Simulating storage services for testing:

```javascript
const { expect } = require('chai');
const sinon = require('sinon');
const AWS = require('aws-sdk');
const StorageService = require('../src/StorageService');

describe("Storage Service", function() {
  let storageService;
  let s3Mock;
  
  beforeEach(function() {
    // Create S3 mock with stubbed methods
    s3Mock = {
      upload: sinon.stub().returns({
        promise: sinon.stub().resolves({
          Location: 'https://bucket.s3.amazonaws.com/file.json',
          Key: 'file.json',
          Bucket: 'bucket'
        })
      }),
      getObject: sinon.stub().returns({
        promise: sinon.stub().resolves({
          Body: JSON.stringify({ key: 'value' })
        })
      }),
      deleteObject: sinon.stub().returns({
        promise: sinon.stub().resolves({})
      })
    };
    
    // Stub AWS S3 constructor
    sinon.stub(AWS, 'S3').returns(s3Mock);
    
    // Create service with mocked dependencies
    storageService = new StorageService({
      region: 'us-east-1',
      bucket: 'bucket'
    });
  });
  
  afterEach(function() {
    sinon.restore();
  });
  
  it("uploads files to storage", async function() {
    // Test data
    const filename = 'test.json';
    const data = { test: 'data' };
    
    // Upload file
    const result = await storageService.uploadFile(filename, data);
    
    // Verify result
    expect(result).to.have.property('location', 'https://bucket.s3.amazonaws.com/file.json');
    expect(result).to.have.property('key', 'file.json');
    
    // Verify S3 was called correctly
    expect(s3Mock.upload.calledOnce).to.be.true;
    
    const uploadArgs = s3Mock.upload.firstCall.args[0];
    expect(uploadArgs).to.have.property('Bucket', 'bucket');
    expect(uploadArgs).to.have.property('Key', filename);
    expect(uploadArgs).to.have.property('Body');
    expect(uploadArgs).to.have.property('ContentType', 'application/json');
  });
  
  it("retrieves files from storage", async function() {
    // Retrieve file
    const result = await storageService.getFile('file.json');
    
    // Verify result
    expect(result).to.deep.equal({ key: 'value' });
    
    // Verify S3 was called correctly
    expect(s3Mock.getObject.calledOnce).to.be.true;
    expect(s3Mock.getObject.firstCall.args[0]).to.deep.equal({
      Bucket: 'bucket',
      Key: 'file.json'
    });
  });
  
  it("deletes files from storage", async function() {
    // Delete file
    await storageService.deleteFile('file.json');
    
    // Verify S3 was called correctly
    expect(s3Mock.deleteObject.calledOnce).to.be.true;
    expect(s3Mock.deleteObject.firstCall.args[0]).to.deep.equal({
      Bucket: 'bucket',
      Key: 'file.json'
    });
  });
  
  it("handles storage errors gracefully", async function() {
    // Setup mock to throw error
    s3Mock.getObject.returns({
      promise: sinon.stub().rejects(new Error('File not found'))
    });
    
    // Test error handling
    try {
      await storageService.getFile('non-existent.json');
      expect.fail('Should have thrown an error');
    } catch (error) {
      expect(error.message).to.include('File not found');
    }
  });
});
```

## Simulation Environments

### Local Blockchain Mocking

Creating and configuring test blockchain environments:

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

describe("Local Blockchain Environment", function() {
  let accounts;
  let testToken;
  
  before(async function() {
    // Get test accounts
    accounts = await ethers.getSigners();
    
    // Deploy a test token
    const TestToken = await ethers.getContractFactory("TestToken");
    testToken = await TestToken.deploy("Test Token", "TST");
    await testToken.deployed();
  });
  
  it("simulates token minting", async function() {
    // Initial balance should be zero
    const initialBalance = await testToken.balanceOf(accounts[1].address);
    expect(initialBalance).to.equal(0);
    
    // Mint tokens
    const mintAmount = ethers.utils.parseEther("1000");
    await testToken.mint(accounts[1].address, mintAmount);
    
    // Check new balance
    const newBalance = await testToken.balanceOf(accounts[1].address);
    expect(newBalance).to.equal(mintAmount);
  });
  
  it("simulates block mining", async function() {
    // Get initial block number
    const initialBlock = await ethers.provider.getBlockNumber();
    
    // Mine additional blocks
    for (let i = 0; i < 5; i++) {
      await ethers.provider.send("evm_mine");
    }
    
    // Check new block number
    const newBlock = await ethers.provider.getBlockNumber();
    expect(newBlock).to.be.at.least(initialBlock + 5);
  });
  
  it("simulates time progression", async function() {
    // Get current block
    const block = await ethers.provider.getBlock();
    const initialTimestamp = block.timestamp;
    
    // Advance time by 7 days
    const secondsIn7Days = 7 * 24 * 60 * 60;
    await ethers.provider.send("evm_increaseTime", [secondsIn7Days]);
    await ethers.provider.send("evm_mine");
    
    // Get new block timestamp
    const newBlock = await ethers.provider.getBlock();
    expect(newBlock.timestamp).to.be.at.least(initialTimestamp + secondsIn7Days);
  });
  
  it("simulates account impersonation", async function() {
    // Create a contract that restricts certain actions to an administrator
    const RestrictedContract = await ethers.getContractFactory("RestrictedContract");
    const restrictedContract = await RestrictedContract.deploy();
    await restrictedContract.deployed();
    
    // Set an admin address
    const adminAddress = "0x1234567890123456789012345678901234567890";
    await restrictedContract.setAdmin(adminAddress);
    
    // Normal account should not have access
    await expect(
      restrictedContract.connect(accounts[2]).adminOnlyFunction()
    ).to.be.revertedWith("Not admin");
    
    // Impersonate the admin account
    await hre.network.provider.request({
      method: "hardhat_impersonateAccount",
      params: [adminAddress],
    });
    
    // Get a signer for the admin
    const adminSigner = await ethers.getSigner(adminAddress);
    
    // Fund the account so it can pay for gas
    await accounts[0].sendTransaction({
      to: adminAddress,
      value: ethers.utils.parseEther("1.0")
    });
    
    // Now admin can access restricted functions
    await restrictedContract.connect(adminSigner).adminOnlyFunction();
    
    // Stop impersonating
    await hre.network.provider.request({
      method: "hardhat_stopImpersonatingAccount",
      params: [adminAddress],
    });
  });
});
```

### Mainnet Forking

Testing against a forked mainnet state:

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

// This test requires hardhat.config.js to be configured with forking enabled:
// networks: {
//   hardhat: {
//     forking: {
//       url: `https://eth-mainnet.alchemyapi.io/v2/${ALCHEMY_API_KEY}`,
//       blockNumber: 15000000,
//     },
//   },
// },

describe("Mainnet Forking Tests", function() {
  // Known addresses from mainnet
  const USDT_ADDRESS = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
  const USDC_ADDRESS = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
  const AAVE_LENDING_POOL = "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9";
  const RICH_ACCOUNT = "0xF977814e90dA44bFA03b6295A0616a897441aceC"; // Binance wallet
  
  let usdt;
  let usdc;
  let accounts;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Connect to existing mainnet contracts
    usdt = await ethers.getContractAt("IERC20", USDT_ADDRESS);
    usdc = await ethers.getContractAt("IERC20", USDC_ADDRESS);
  });
  
  it("should access real USDT contract state", async function() {
    // Check real mainnet token details
    expect(await usdt.name()).to.equal("Tether USD");
    expect(await usdt.symbol()).to.equal("USDT");
    expect(await usdt.decimals()).to.equal(6);
  });
  
  it("impersonates a whale account to use their tokens", async function() {
    // Verify the whale has tokens
    const whaleUsdtBalance = await usdt.balanceOf(RICH_ACCOUNT);
    expect(whaleUsdtBalance).to.be.gt(0);
    console.log(`Whale USDT balance: ${ethers.utils.formatUnits(whaleUsdtBalance, 6)}`);
    
    // Impersonate the account
    await hre.network.provider.request({
      method: "hardhat_impersonateAccount",
      params: [RICH_ACCOUNT],
    });
    
    // Get a signer for the whale
    const whaleSigner = await ethers.getSigner(RICH_ACCOUNT);
    
    // Transfer some USDT to our test account
    const transferAmount = 1000000000; // 1000 USDT (6 decimals)
    await usdt.connect(whaleSigner).transfer(accounts[0].address, transferAmount);
    
    // Verify transfer succeeded
    const newBalance = await usdt.balanceOf(accounts[0].address);
    expect(newBalance).to.equal(transferAmount);
    
    // Stop impersonating
    await hre.network.provider.request({
      method: "hardhat_stopImpersonatingAccount",
      params: [RICH_ACCOUNT],
    });
  });
  
  it("interacts with mainnet DeFi protocols", async function() {
    // Deploy a contract that integrates with AAVE
    const AaveIntegrator = await ethers.getContractFactory("AaveIntegrator");
    const aaveIntegrator = await AaveIntegrator.deploy(AAVE_LENDING_POOL);
    await aaveIntegrator.deployed();
    
    // Get whale account with lots of USDC
    await hre.network.provider.request({
      method: "hardhat_impersonateAccount",
      params: [RICH_ACCOUNT],
    });
    const whaleSigner = await ethers.getSigner(RICH_ACCOUNT);
    
    // Transfer USDC to our test contract
    const depositAmount = 10000000000; // 10,000 USDC (6 decimals)
    await usdc.connect(whaleSigner).transfer(aaveIntegrator.address, depositAmount);
    
    // Deposit to AAVE through our contract
    await aaveIntegrator.depositToAave(USDC_ADDRESS, depositAmount);
    
    // Check that our contract now has aUSDC
    const aUsdcAddress = await aaveIntegrator.getATokenAddress(USDC_ADDRESS);
    const aUsdc = await ethers.getContractAt("IERC20", aUsdcAddress);
    const aUsdcBalance = await aUsdc.balanceOf(aaveIntegrator.address);
    
    expect(aUsdcBalance).to.be.gt(0);
    console.log(`aUSDC balance after deposit: ${ethers.utils.formatUnits(aUsdcBalance, 6)}`);
    
    // We could also test withdrawals, but this is enough to demonstrate the concept
    await hre.network.provider.request({
      method: "hardhat_stopImpersonatingAccount",
      params: [RICH_ACCOUNT],
    });
  });
  
  it("modifies mainnet state for testing edge cases", async function() {
    // Deploy a contract that depends on Chainlink price feeds
    const ETH_USD_PRICE_FEED = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";
    const PriceConsumer = await ethers.getContractFactory("PriceConsumer");
    const priceConsumer = await PriceConsumer.deploy(ETH_USD_PRICE_FEED);
    await priceConsumer.deployed();
    
    // Get current ETH price
    const currentPrice = await priceConsumer.getLatestPrice();
    console.log(`Current ETH price: $${ethers.utils.formatUnits(currentPrice, 8)}`);
    
    // Get the Chainlink Aggregator contract
    const aggregator = await ethers.getContractAt("IAggregatorV3", ETH_USD_PRICE_FEED);
    
    // Find the storage slot for the latest round data
    // This is a simplified approach - in reality, you'd need to analyze the contract storage layout
    // Here we're just demonstrating the concept
    
    // The actual implementation would require:
    // 1. Analyzing the storage layout of the Chainlink Aggregator
    // 2. Finding the correct slot for the latest round data
    // 3. Creating the appropriate data structure to modify it
    
    // For this example, we're using the impersonation approach instead
    // Let's impersonate the Chainlink oracle and submit a new price answer
    
    // Get the address that can update the price feed
    const aggregatorOwner = await aggregator.owner();
    
    // Impersonate the owner (this is just an example and might not work 
    // as Chainlink aggregators have complex access controls)
    await hre.network.provider.request({
      method: "hardhat_impersonateAccount",
      params: [aggregatorOwner],
    });
    
    // Fund the owner account
    await accounts[0].sendTransaction({
      to: aggregatorOwner,
      value: ethers.utils.parseEther("1.0")
    });
    
    // Get signer for owner
    const ownerSigner = await ethers.getSigner(aggregatorOwner);
    
    // Now in a real test we'd call the appropriate function to update the price data
    // Since it's too complex for this example, we'll just demonstrate the concept 
    // with a contract that we control
    
    // Deploy a mock price feed
    const MockPriceFeed = await ethers.getContractFactory("MockPriceFeed");
    const mockPriceFeed = await MockPriceFeed.deploy();
    await mockPriceFeed.deployed();
    
    // Set a mock price - ETH crashed to $500
    const crashPrice = ethers.utils.parseUnits("500", 8); // 8 decimals like Chainlink
    await mockPriceFeed.setPrice(crashPrice);
    
    // Update the price consumer to use our mock
    await priceConsumer.setPriceFeed(mockPriceFeed.address);
    
    // Get the new price
    const newPrice = await priceConsumer.getLatestPrice();
    expect(newPrice).to.equal(crashPrice);
    console.log(`New ETH price: $${ethers.utils.formatUnits(newPrice, 8)}`);
    
    // Clean up impersonation
    await hre.network.provider.request({
      method: "hardhat_stopImpersonatingAccount",
      params: [aggregatorOwner],
    });
  });
});
```

### Customized Test Networks

Creating specialized networks for specific test cases:

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

// This test requires hardhat.config.js to include a custom network configuration:
// networks: {
//   hardhat: {
//     hardfork: "london",
//     initialBaseFeePerGas: 1000000000,
//     accounts: {
//       mnemonic: "test test test test test test test test test test test junk",
//       accountsBalance: "10000000000000000000000" // 10000 ETH
//     },
//     mining: {
//       auto: false,
//       interval: 5000 // ms
//     }
//   },
// }

describe("Custom Network Tests", function() {
  let accounts;
  let highGasContract;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy a contract that uses lots of gas
    const HighGasContract = await ethers.getContractFactory("HighGasOperations");
    highGasContract = await HighGasContract.deploy();
    await highGasContract.deployed();
  });
  
  it("tests behavior under congested network conditions", async function() {
    // Simulate network congestion by increasing base fee
    await network.provider.send("hardhat_setNextBlockBaseFeePerGas", [
      "0x" + (1000000000000).toString(16) // 1000 gwei
    ]);
    
    // Set gas price to auto so it adjusts to base fee
    await network.provider.send("evm_setAutomine", [true]);
    
    // Execute a transaction with default gas settings
    const tx = await highGasContract.performExpensiveOperation(100);
    const receipt = await tx.wait();
    
    // Log gas usage under congested conditions
    console.log(`Gas used under congestion: ${receipt.gasUsed.toString()}`);
    console.log(`Effective gas price: ${ethers.utils.formatUnits(receipt.effectiveGasPrice, 'gwei')} gwei`);
    
    // Reset base fee to normal
    await network.provider.send("hardhat_setNextBlockBaseFeePerGas", [
      "0x" + (1000000000).toString(16) // 1 gwei
    ]);
  });
  
  it("tests behavior with custom block timing", async function() {
    // Switch to manual mining
    await network.provider.send("evm_setAutomine", [false]);
    
    // Send a transaction but don't mine it immediately
    const pendingTxPromise = highGasContract.storeValue(42);
    
    // Check that the value hasn't been updated yet
    const currentValue = await highGasContract.storedValue();
    expect(currentValue).to.not.equal(42);
    
    // Mine a block to include the transaction
    await network.provider.send("evm_mine");
    
    // Wait for the transaction to complete
    await pendingTxPromise;
    
    // Now the value should be updated
    const newValue = await highGasContract.storedValue();
    expect(newValue).to.equal(42);
    
    // Switch back to automatic mining
    await network.provider.send("evm_setAutomine", [true]);
  });
  
  it("tests contract behavior with specific gas limits", async function() {
    // Set a very low block gas limit to simulate network constraints
    await network.provider.send("evm_setBlockGasLimit", [300000]);
    
    // A moderately complex operation should still work
    await highGasContract.performModerateOperation();
    
    // But an expensive operation should fail
    await expect(
      highGasContract.performExpensiveOperation(1000)
    ).to.be.reverted; // Should fail with out of gas error
    
    // Reset block gas limit to normal
    await network.provider.send("evm_setBlockGasLimit", [30000000]);
  });
  
  it("tests contract during network upgrades", async function() {
    // Test with pre-London rules (no EIP-1559)
    await network.provider.send("hardhat_setHardfork", ["berlin"]);
    
    // Legacy transaction should work fine
    const legacyTx = await accounts[0].sendTransaction({
      to: accounts[1].address,
      value: ethers.utils.parseEther("1.0"),
      gasPrice: ethers.utils.parseUnits("20", "gwei")
    });
    
    const legacyReceipt = await legacyTx.wait();
    expect(legacyReceipt.status).to.equal(1);
    
    // Switch to London rules (EIP-1559 active)
    await network.provider.send("hardhat_setHardfork", ["london"]);
    
    // EIP-1559 transaction should work
    const eip1559Tx = await accounts[0].sendTransaction({
      to: accounts[1].address,
      value: ethers.utils.parseEther("1.0"),
      maxFeePerGas: ethers.utils.parseUnits("50", "gwei"),
      maxPriorityFeePerGas: ethers.utils.parseUnits("2", "gwei")
    });
    
    const eip1559Receipt = await eip1559Tx.wait();
    expect(eip1559Receipt.status).to.equal(1);
  });
});
```

## Test Doubles for Blockchain Components

### Mock Contracts

Creating mock versions of contracts for testing:

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

describe("Mock Contracts", function() {
  let accounts;
  let mockToken;
  let mockOracle;
  let tokenUser;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy mock token
    const MockToken = await ethers.getContractFactory("MockERC20");
    mockToken = await MockToken.deploy(
      "Mock Token",
      "MOCK",
      18
    );
    await mockToken.deployed();
    
    // Deploy mock price oracle
    const MockOracle = await ethers.getContractFactory("MockPriceOracle");
    mockOracle = await MockOracle.deploy();
    await mockOracle.deployed();
    
    // Deploy contract that uses token and oracle
    const TokenUser = await ethers.getContractFactory("TokenUser");
    tokenUser = await TokenUser.deploy(
      mockToken.address,
      mockOracle.address
    );
    await tokenUser.deployed();
  });
  
  it("uses mock token to simulate transfers", async function() {
    // Configure mock token
    await mockToken.mint(accounts[1].address, ethers.utils.parseEther("1000"));
    
    // Check initial balances
    expect(await mockToken.balanceOf(accounts[1].address)).to.equal(ethers.utils.parseEther("1000"));
    expect(await mockToken.balanceOf(accounts[2].address)).to.equal(0);
    
    // Perform transfer
    await mockToken.connect(accounts[1]).transfer(accounts[2].address, ethers.utils.parseEther("500"));
    
    // Verify balances after transfer
    expect(await mockToken.balanceOf(accounts[1].address)).to.equal(ethers.utils.parseEther("500"));
    expect(await mockToken.balanceOf(accounts[2].address)).to.equal(ethers.utils.parseEther("500"));
  });
  
  it("configures mock token to simulate failures", async function() {
    // Configure mock to fail on transfers
    await mockToken.setTransferShouldRevert(true);
    
    // Transfer should now fail
    await expect(
      mockToken.connect(accounts[1]).transfer(accounts[2].address, ethers.utils.parseEther("100"))
    ).to.be.revertedWith("Mock ERC20: transfer reverted");
    
    // Balances should remain unchanged
    expect(await mockToken.balanceOf(accounts[1].address)).to.equal(ethers.utils.parseEther("500"));
    expect(await mockToken.balanceOf(accounts[2].address)).to.equal(ethers.utils.parseEther("500"));
    
    // Reset mock behavior
    await mockToken.setTransferShouldRevert(false);
    
    // Transfer should succeed again
    await mockToken.connect(accounts[1]).transfer(accounts[2].address, ethers.utils.parseEther("100"));
    expect(await mockToken.balanceOf(accounts[1].address)).to.equal(ethers.utils.parseEther("400"));
  });
  
  it("uses mock oracle to simulate price feeds", async function() {
    // Set mock price data
    await mockOracle.setPrice(
      mockToken.address,
      ethers.utils.parseUnits("2.5", 8) // $2.50 with 8 decimals
    );
    
    // Use the contract that depends on the price oracle
    const tokenPrice = await tokenUser.getTokenPrice();
    expect(tokenPrice).to.equal(ethers.utils.parseUnits("2.5", 8));
    
    // Calculate value using the mock price
    const valueInEth = await tokenUser.calculateTokenValue(ethers.utils.parseEther("100"));
    
    // 100 tokens @ $2.50 = $250
    expect(valueInEth).to.equal(ethers.utils.parseUnits("250", 8));
  });
  
  it("uses mock oracle to test price impact scenarios", async function() {
    // Test how system handles price changes
    
    // Set initial price
    await mockOracle.setPrice(
      mockToken.address,
      ethers.utils.parseUnits("2.5", 8)
    );
    
    // User buys tokens at this price
    await tokenUser.connect(accounts[1]).buyTokens({ value: ethers.utils.parseEther("1.0") });
    const initialBalance = await mockToken.balanceOf(accounts[1].address);
    
    // Price crashes - simulate market crash
    await mockOracle.setPrice(
      mockToken.address,
      ethers.utils.parseUnits("1.0", 8) // Price drops to $1.00
    );
    
    // User tries to sell tokens after crash
    await mockToken.connect(accounts[1]).approve(tokenUser.address, initialBalance);
    await tokenUser.connect(accounts[1]).sellTokens(initialBalance);
    
    // Verify user got less ETH back due to price decrease
    // Specific assertions would depend on the TokenUser implementation
  });
  
  it("simulates complex mock contract interactions", async function() {
    // Deploy mock lending protocol
    const MockLendingProtocol = await ethers.getContractFactory("MockLendingProtocol");
    const mockLending = await MockLendingProtocol.deploy(mockToken.address);
    await mockLending.deployed();
    
    // Set up interest rate in mock
    await mockLending.setInterestRate(500); // 5.00%
    expect(await mockLending.getInterestRate()).to.equal(500);
    
    // Mint tokens for testing
    await mockToken.mint(accounts[3].address, ethers.utils.parseEther("10000"));
    await mockToken.connect(accounts[3]).approve(mockLending.address, ethers.utils.parseEther("10000"));
    
    // Deposit tokens to lending protocol
    await mockLending.connect(accounts[3]).deposit(ethers.utils.parseEther("10000"));
    expect(await mockLending.getDepositBalance(accounts[3].address)).to.equal(ethers.utils.parseEther("10000"));
    
    // Fast forward time to accrue interest
    await ethers.provider.send("evm_increaseTime", [365 * 24 * 60 * 60]); // 1 year
    await ethers.provider.send("evm_mine");
    
    // Update accrued interest in mock
    await mockLending.updateInterest();
    
    // Check new balance with interest
    const newBalance = await mockLending.getDepositBalance(accounts[3].address);
    
    // Should be original amount + 5% interest
    const expectedBalance = ethers.utils.parseEther("10500"); // 10000 + 5%
    expect(newBalance).to.be.closeTo(expectedBalance, ethers.utils.parseEther("0.01"));
  });
});
```

Example Mock ERC20 Contract:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
    uint8 private _decimals;
    bool public transferShouldRevert;
    bool public approveShouldRevert;
    bool public transferFromShouldRevert;
    
    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_
    ) ERC20(name_, symbol_) {
        _decimals = decimals_;
    }
    
    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }
    
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
    
    function burn(address from, uint256 amount) external {
        _burn(from, amount);
    }
    
    function setTransferShouldRevert(bool shouldRevert) external {
        transferShouldRevert = shouldRevert;
    }
    
    function setApproveShouldRevert(bool shouldRevert) external {
        approveShouldRevert = shouldRevert;
    }
    
    function setTransferFromShouldRevert(bool shouldRevert) external {
        transferFromShouldRevert = shouldRevert;
    }
    
    function transfer(address to, uint256 amount) public virtual override returns (bool) {
        if (transferShouldRevert) {
            revert("Mock ERC20: transfer reverted");
        }
        return super.transfer(to, amount);
    }
    
    function approve(address spender, uint256 amount) public virtual override returns (bool) {
        if (approveShouldRevert) {
            revert("Mock ERC20: approve reverted");
        }
        return super.approve(spender, amount);
    }
    
    function transferFrom(address from, address to, uint256 amount) public virtual override returns (bool) {
        if (transferFromShouldRevert) {
            revert("Mock ERC20: transferFrom reverted");
        }
        return super.transferFrom(from, to, amount);
    }
}
```

### Blockchain Service Mocking

Mocking blockchain services and oracles:

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

describe("Blockchain Service Mocking", function() {
  let accounts;
  let mockRandomnessProvider;
  let mockCrosschainBridge;
  let mockStakingService;
  let dappContract;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy mock services
    const MockRandomness = await ethers.getContractFactory("MockRandomnessProvider");
    mockRandomnessProvider = await MockRandomness.deploy();
    await mockRandomnessProvider.deployed();
    
    const MockBridge = await ethers.getContractFactory("MockCrosschainBridge");
    mockCrosschainBridge = await MockBridge.deploy();
    await mockCrosschainBridge.deployed();
    
    const MockStaking = await ethers.getContractFactory("MockStakingService");
    mockStakingService = await MockStaking.deploy();
    await mockStakingService.deployed();
    
    // Deploy dapp contract that uses these services
    const DappContract = await ethers.getContractFactory("DappContract");
    dappContract = await DappContract.deploy(
      mockRandomnessProvider.address,
      mockCrosschainBridge.address,
      mockStakingService.address
    );
    await dappContract.deployed();
  });
  
  describe("Randomness Service", function() {
    it("tests contract behavior with predetermined random values", async function() {
      // Set specific random value in the mock
      await mockRandomnessProvider.setRandomNumber(42);
      
      // Request randomness
      await dappContract.requestRandomNumber();
      
      // Simulate callback from VRF
      await mockRandomnessProvider.fulfillRandomnessRequest(dappContract.address);
      
      // Check that our dapp received the random number
      const randomResult = await dappContract.getLastRandomNumber();
      expect(randomResult).to.equal(42);
    });
    
    it("tests contract handling of randomness failures", async function() {
      // Configure mock to simulate failure
      await mockRandomnessProvider.setShouldFail(true);
      
      // Request should still work
      await dappContract.requestRandomNumber();
      
      // But fulfillment should fail
      await expect(
        mockRandomnessProvider.fulfillRandomnessRequest(dappContract.address)
      ).to.be.revertedWith("Mock randomness fulfillment failed");
      
      // Reset mock
      await mockRandomnessProvider.setShouldFail(false);
    });
  });
  
  describe("Cross-chain Bridge", function() {
    it("tests cross-chain token transfers", async function() {
      // Set up the bridge mock
      await mockCrosschainBridge.setRemoteToken("polygon", "0x1234567890123456789012345678901234567890");
      
      // Initiate a cross-chain transfer
      await dappContract.sendCrosschainToken(
        "polygon",
        accounts[1].address,
        ethers.utils.parseEther("100")
      );
      
      // Verify the bridge was called correctly
      const lastTransfer = await mockCrosschainBridge.getLastTransfer();
      expect(lastTransfer.chainId).to.equal("polygon");
      expect(lastTransfer.recipient).to.equal(accounts[1].address);
      expect(lastTransfer.amount).to.equal(ethers.utils.parseEther("100"));
    });
    
    it("tests receiving cross-chain messages", async function() {
      // Simulate a cross-chain message being received
      await mockCrosschainBridge.simulateMessageFromChain(
        "avalanche",
        "0x9876543210987654321098765432109876543210",
        ethers.utils.defaultAbiCoder.encode(
          ["string", "uint256"],
          ["Hello from Avalanche", 12345]
        )
      );
      
      // Process the message in our dapp
      await dappContract.processCrosschainMessages();
      
      // Verify the message was processed
      const lastMessage = await dappContract.getLastCrosschainMessage();
      expect(lastMessage.sourceChain).to.equal("avalanche");
      expect(lastMessage.sender).to.equal("0x9876543210987654321098765432109876543210");
      expect(lastMessage.decodedString).to.equal("Hello from Avalanche");
      expect(lastMessage.decodedNumber).to.equal(12345);
    });
  });
  
  describe("Staking Service", function() {
    it("tests rewards calculation with mock staking service", async function() {
      // Configure mock staking service
      await mockStakingService.setStakedAmount(accounts[2].address, ethers.utils.parseEther("5000"));
      await mockStakingService.setRewardRate(500); // 5% annual return
      
      // Fast forward time for rewards accrual
      await ethers.provider.send("evm_increaseTime", [30 * 24 * 60 * 60]); // 30 days
      await ethers.provider.send("evm_mine");
      
      // Update rewards in mock
      await mockStakingService.updateRewards(accounts[2].address);
      
      // Check rewards through the dapp
      const rewards = await dappContract.getUserStakingRewards(accounts[2].address);
      
      // Expected rewards: 5000 * 5% * (30/365) ≈ 20.55 tokens
      const expectedRewards = ethers.utils.parseEther("20.55");
      expect(rewards).to.be.closeTo(expectedRewards, ethers.utils.parseEther("0.01"));
    });
    
    it("tests staking and unstaking functionality", async function() {
      // Mint some tokens for the test
      const TestToken = await ethers.getContractFactory("TestToken");
      const testToken = await TestToken.deploy("Test Token", "TST");
      await testToken.deployed();
      await testToken.mint(accounts[3].address, ethers.utils.parseEther("1000"));
      
      // Configure mock staking service to use this token
      await mockStakingService.setStakingToken(testToken.address);
      
      // Approve tokens for staking
      await testToken.connect(accounts[3]).approve(dappContract.address, ethers.utils.parseEther("1000"));
      
      // Stake tokens through the dapp
      await dappContract.connect(accounts[3]).stakeTokens(ethers.utils.parseEther("500"));
      
      // Verify staking happened in the mock
      expect(await mockStakingService.getStakedAmount(accounts[3].address))
        .to.equal(ethers.utils.parseEther("500"));
      
      // Unstake some tokens
      await dappContract.connect(accounts[3]).unstakeTokens(ethers.utils.parseEther("200"));
      
      // Verify unstaking happened in the mock
      expect(await mockStakingService.getStakedAmount(accounts[3].address))
        .to.equal(ethers.utils.parseEther("300"));
    });
  });
});
```

### Oracle Simulation

Mocking blockchain oracles:

```javascript
const { expect } = require('chai');
const { ethers } = require('hardhat');

describe("Oracle Simulation", function() {
  let accounts;
  let mockPriceOracle;
  let mockWeatherOracle;
  let mockDataFeed;
  let oracleConsumer;
  
  before(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy mock oracles
    const MockPriceOracle = await ethers.getContractFactory("MockPriceOracle");
    mockPriceOracle = await MockPriceOracle.deploy();
    await mockPriceOracle.deployed();
    
    const MockWeatherOracle = await ethers.getContractFactory("MockWeatherOracle");
    mockWeatherOracle = await MockWeatherOracle.deploy();
    await mockWeatherOracle.deployed();
    
    const MockDataFeed = await ethers.getContractFactory("MockDataFeed");
    mockDataFeed = await MockDataFeed.deploy();
    await mockDataFeed.deployed();
    
    // Deploy consumer contract
    const OracleConsumer = await ethers.getContractFactory("OracleConsumer");
    oracleConsumer = await OracleConsumer.deploy(
      mockPriceOracle.address,
      mockWeatherOracle.address,
      mockDataFeed.address
    );
    await oracleConsumer.deployed();
  });
  
  describe("Price Oracle", function() {
    it("provides price data to consumer contracts", async function() {
      // Set up price data in the mock
      await mockPriceOracle.setPrice(
        "ETH",
        ethers.utils.parseUnits("1800.50", 8) // $1800.50 with 8 decimal places
      );
      
      await mockPriceOracle.setPrice(
        "BTC",
        ethers.utils.parseUnits("28750.75", 8) // $28750.75 with 8 decimal places
      );
      
      // Get prices through the consumer contract
      const ethPrice = await oracleConsumer.getAssetPrice("ETH");
      const btcPrice = await oracleConsumer.getAssetPrice("BTC");
      
      // Verify prices
      expect(ethPrice).to.equal(ethers.utils.parseUnits("1800.50", 8));
      expect(btcPrice).to.equal(ethers.utils.parseUnits("28750.75", 8));
    });
    
    it("tests price-dependent business logic", async function() {
      // Set ETH price to $2000
      await mockPriceOracle.setPrice(
        "ETH",
        ethers.utils.parseUnits("2000.00", 8)
      );
      
      // Set premium threshold in consumer to $1900
      await oracleConsumer.setPremiumThreshold(ethers.utils.parseUnits("1900.00", 8));
      
      // Check if price is above premium threshold
      expect(await oracleConsumer.isPremiumPrice("ETH")).to.be.true;
      
      // Lower ETH price to $1800
      await mockPriceOracle.setPrice(
        "ETH",
        ethers.utils.parseUnits("1800.00", 8)
      );
      
      // Check if price is now below premium threshold
      expect(await oracleConsumer.isPremiumPrice("ETH")).to.be.false;
    });
  });
  
  describe("Weather Oracle", function() {
    it("provides weather data to consumer contracts", async function() {
      // Set up weather data in the mock
      await mockWeatherOracle.setTemperature("NYC", 25); // 25°C
      await mockWeatherOracle.setPrecipitation("NYC", 80); // 80% chance of rain
      
      // Get weather data through the consumer
      const [temperature, precipitation] = await oracleConsumer.getWeatherData("NYC");
      
      // Verify weather data
      expect(temperature).to.equal(25);
      expect(precipitation).to.equal(80);
    });
    
    it("tests weather-dependent business logic", async function() {
      // Configure consumer logic
      await oracleConsumer.setWeatherThresholds(30, 60); // >30°C or >60% rain is "bad weather"
      
      // Test with good weather
      await mockWeatherOracle.setTemperature("MIA", 28);
      await mockWeatherOracle.setPrecipitation("MIA", 40);
      
      expect(await oracleConsumer.isBadWeather("MIA")).to.be.false;
      
      // Test with high temperature (bad weather)
      await mockWeatherOracle.setTemperature("MIA", 32);
      await mockWeatherOracle.setPrecipitation("MIA", 40);
      
      expect(await oracleConsumer.isBadWeather("MIA")).to.be.true;
      
      // Test with high precipitation (bad weather)
      await mockWeatherOracle.setTemperature("MIA", 28);
      await mockWeatherOracle.setPrecipitation("MIA", 70);
      
      expect(await oracleConsumer.isBadWeather("MIA")).to.be.true;
    });
  });
  
  describe("Data Feed Oracle", function() {
    it("handles complex data feeds", async function() {
      // Set up complex data in mock
      const stockData = {
        symbol: "AAPL",
        price: "18750",  // $187.50 with 2 decimal precision
        changePercent: "250",  // 2.50% with 2 decimal precision
        volume: "12435678",
        marketCap: "298750000000" // $2.9875T
      };
      
      await mockDataFeed.setJsonData(
        "stocks",
        JSON.stringify(stockData)
      );
      
      // Request data through the consumer
      await oracleConsumer.requestDataFeed("stocks");
      
      // Simulate oracle callback
      await mockDataFeed.fulfillDataRequest(
        oracleConsumer.address,
        "stocks",
        JSON.stringify(stockData)
      );
      
      // Get the parsed data
      const parsedData = await oracleConsumer.getLastStockData();
      
      // Verify data was correctly received and parsed
      expect(parsedData.symbol).to.equal("AAPL");
      expect(parsedData.price).to.equal(18750);
      expect(parsedData.changePercent).to.equal(250);
      expect(parsedData.volume).to.equal(12435678);
      expect(parsedData.marketCap).to.equal("298750000000");
    });
    
    it("tests handling of malformatted data", async function() {
      // Set up malformatted data
      const badData = "{not valid json";
      
      await mockDataFeed.setJsonData(
        "bad-format",
        badData
      );
      
      // Request the bad data
      await oracleConsumer.requestDataFeed("bad-format");
      
      // Simulate oracle callback with bad data
      await mockDataFeed.fulfillDataRequest(
        oracleConsumer.address,
        "bad-format",
        badData
      );
      
      // Check that the consumer handled the error
      expect(await oracleConsumer.hasDataError()).to.be.true;
      expect(await oracleConsumer.getLastErrorFeed()).to.equal("bad-format");
    });
    
    it("tests handling of delayed or missing oracle responses", async function() {
      // Request data but don't fulfill the request yet
      await oracleConsumer.requestDataFeed("delayed-data");
      
      // Set timeout for handling delayed data
      await oracleConsumer.setResponseTimeout(60); // 60 seconds timeout
      
      // Fast forward time past the timeout
      await ethers.provider.send("evm_increaseTime", [120]); // 120 seconds
      await ethers.provider.send("evm_mine");
      
      // Check request status
      expect(await oracleConsumer.isRequestTimedOut("delayed-data")).to.be.true;
      
      // Simulate late oracle response
      const lateData = {
        message: "Sorry for the delay",
        timestamp: Math.floor(Date.now() / 1000)
      };
      
      // This should be rejected due to timeout
      await mockDataFeed.fulfillDataRequest(
        oracleConsumer.address,
        "delayed-data",
        JSON.stringify(lateData)
      );
      
      // Check that the late data was rejected
      expect(await oracleConsumer.getLastErrorFeed()).to.equal("delayed-data");
      expect(await oracleConsumer.getLastErrorReason()).to.equal("Request timed out");
    });
  });
});
```

## Conclusion

Mock systems are crucial for comprehensive testing of blockchain applications, allowing developers to isolate components, simulate various scenarios, and test edge cases that would be difficult or expensive to reproduce in real environments. By using the techniques covered in this chapter—from basic API mocking to full simulation environments and test doubles—ProzChain developers can build more reliable, robust applications that behave predictably across a wide range of conditions.

When implementing mock systems:

1. **Design for testability**: Structure components with interfaces that are easy to mock
2. **Keep mocks simple**: Create focused mocks that simulate only the behavior needed for tests
3. **Test edge cases**: Use mocks to simulate failures, delays, and unusual responses
4. **Balance realism and complexity**: More realistic mocks provide better test coverage but can be harder to maintain

In the next chapter, we'll explore how to integrate these testing techniques into continuous integration pipelines for automated testing and validation.
