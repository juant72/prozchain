# Integration Testing

## Overview

Integration testing verifies that different components of a ProzChain application work together as expected. Unlike unit tests that isolate individual functions or components, integration tests focus on the interactions between multiple contracts, external services, or system layers. This approach is particularly important for blockchain applications, where complex contract interactions, economic systems, and cross-contract calls create emergent behaviors that can't be fully tested in isolation.

This chapter explores techniques for effective integration testing of ProzChain applications, from contract interactions to external system integration.

## Testing Component Interactions

### Contract-to-Contract Interactions

Testing how contracts interact with each other:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Token and Staking Contract Integration", function() {
  let token;
  let stakingContract;
  let owner;
  let user;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Deploy token contract
    const Token = await ethers.getContractFactory("Token");
    token = await Token.deploy("Staking Token", "STK");
    
    // Deploy staking contract that references the token
    const StakingContract = await ethers.getContractFactory("StakingContract");
    stakingContract = await StakingContract.deploy(token.address);
    
    // Mint tokens to user
    await token.mint(user.address, ethers.utils.parseEther("1000"));
    
    // Approve staking contract to spend user tokens
    await token.connect(user).approve(stakingContract.address, ethers.utils.parseEther("1000"));
  });
  
  it("allows users to stake tokens", async function() {
    // Initial state
    expect(await stakingContract.balanceOf(user.address)).to.equal(0);
    const initialTokenBalance = await token.balanceOf(user.address);
    
    // Perform staking
    const stakeAmount = ethers.utils.parseEther("100");
    await stakingContract.connect(user).stake(stakeAmount);
    
    // Verify staking occurred correctly
    expect(await stakingContract.balanceOf(user.address)).to.equal(stakeAmount);
    expect(await token.balanceOf(user.address)).to.equal(initialTokenBalance.sub(stakeAmount));
    expect(await token.balanceOf(stakingContract.address)).to.equal(stakeAmount);
  });
  
  it("allows users to unstake tokens", async function() {
    // Setup - stake tokens first
    const stakeAmount = ethers.utils.parseEther("100");
    await stakingContract.connect(user).stake(stakeAmount);
    
    const initialTokenBalance = await token.balanceOf(user.address);
    
    // Perform unstaking
    await stakingContract.connect(user).unstake(stakeAmount);
    
    // Verify unstaking worked correctly
    expect(await stakingContract.balanceOf(user.address)).to.equal(0);
    expect(await token.balanceOf(user.address)).to.equal(initialTokenBalance.add(stakeAmount));
    expect(await token.balanceOf(stakingContract.address)).to.equal(0);
  });
  
  it("calculates rewards correctly", async function() {
    // Setup - stake tokens
    const stakeAmount = ethers.utils.parseEther("100");
    await stakingContract.connect(user).stake(stakeAmount);
    
    // Simulate time passing (for reward accrual)
    await ethers.provider.send("evm_increaseTime", [86400]); // 1 day
    await ethers.provider.send("evm_mine");
    
    // Check rewards
    const expectedRewards = await stakingContract.calculateRewards(user.address);
    expect(expectedRewards).to.be.gt(0);
    
    // Claim rewards
    await stakingContract.connect(user).claimRewards();
    
    // Verify rewards were transferred correctly
    const rewardTokenBalance = await token.balanceOf(user.address);
    expect(rewardTokenBalance).to.be.gt(stakeAmount);
  });
});
```

### Multi-Contract Test Scenarios

Testing interactions across multiple contracts:

```javascript
describe("DeFi Protocol Integration", function() {
  let token;
  let lpToken;
  let exchange;
  let lendingPool;
  let oracle;
  let owner;
  let user;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Deploy mock price oracle
    const MockOracle = await ethers.getContractFactory("MockPriceOracle");
    oracle = await MockOracle.deploy();
    
    // Deploy base token
    const Token = await ethers.getContractFactory("Token");
    token = await Token.deploy("Base Token", "BASE");
    
    // Deploy LP token
    lpToken = await Token.deploy("LP Token", "LP");
    
    // Deploy exchange
    const Exchange = await ethers.getContractFactory("Exchange");
    exchange = await Exchange.deploy(token.address, lpToken.address, oracle.address);
    
    // Deploy lending pool
    const LendingPool = await ethers.getContractFactory("LendingPool");
    lendingPool = await LendingPool.deploy(token.address, oracle.address);
    
    // Setup initial conditions
    await token.mint(owner.address, ethers.utils.parseEther("1000000"));
    await token.transfer(user.address, ethers.utils.parseEther("10000"));
    await token.transfer(exchange.address, ethers.utils.parseEther("100000"));
    await lpToken.mint(exchange.address, ethers.utils.parseEther("100000"));
    
    // Set price in oracle
    await oracle.setPrice(token.address, ethers.utils.parseUnits("100", 8)); // $100 per token
  });
  
  it("allows end-to-end swap and lending flow", async function() {
    // Approve exchange to spend tokens
    await token.connect(user).approve(exchange.address, ethers.utils.parseEther("1000"));
    
    // Swap tokens for LP tokens
    await exchange.connect(user).swapExactTokensForLP(
      ethers.utils.parseEther("100"),
      ethers.utils.parseEther("0") // Min LP out
    );
    
    // Verify swap worked
    expect(await lpToken.balanceOf(user.address)).to.be.gt(0);
    const lpBalance = await lpToken.balanceOf(user.address);
    
    // Approve lending pool to spend LP tokens
    await lpToken.connect(user).approve(lendingPool.address, lpBalance);
    
    // Deposit LP tokens as collateral
    await lendingPool.connect(user).depositCollateral(lpToken.address, lpBalance);
    
    // Verify collateral was accepted
    expect(await lendingPool.getCollateralBalance(user.address, lpToken.address)).to.equal(lpBalance);
    
    // Calculate borrow capacity
    const borrowLimit = await lendingPool.getBorrowLimit(user.address);
    expect(borrowLimit).to.be.gt(0);
    
    // Borrow some tokens (50% of limit)
    const borrowAmount = borrowLimit.div(2);
    await lendingPool.connect(user).borrow(borrowAmount);
    
    // Verify borrowed amount was received
    const finalTokenBalance = await token.balanceOf(user.address);
    expect(finalTokenBalance).to.be.gt(ethers.utils.parseEther("9900")); // Original minus 100 swapped plus loan
    
    // Verify loan was recorded
    expect(await lendingPool.getBorrowBalance(user.address)).to.equal(borrowAmount);
  });
  
  it("handles liquidations when price drops", async function() {
    // Setup a position
    await token.connect(user).approve(exchange.address, ethers.utils.parseEther("5000"));
    await exchange.connect(user).swapExactTokensForLP(
      ethers.utils.parseEther("5000"),
      ethers.utils.parseEther("0")
    );
    
    const lpBalance = await lpToken.balanceOf(user.address);
    await lpToken.connect(user).approve(lendingPool.address, lpBalance);
    await lendingPool.connect(user).depositCollateral(lpToken.address, lpBalance);
    
    const borrowLimit = await lendingPool.getBorrowLimit(user.address);
    await lendingPool.connect(user).borrow(borrowLimit.mul(80).div(100)); // Borrow 80% of limit
    
    // Crash the price by 50%
    await oracle.setPrice(token.address, ethers.utils.parseUnits("50", 8)); // Price drops to $50
    
    // User is now undercollateralized
    expect(await lendingPool.isHealthy(user.address)).to.be.false;
    
    // Liquidator setup
    const liquidator = owner;
    const liquidationAmount = await lendingPool.getBorrowBalance(user.address);
    await token.connect(liquidator).approve(lendingPool.address, liquidationAmount);
    
    // Perform liquidation
    await lendingPool.connect(liquidator).liquidate(user.address, liquidationAmount);
    
    // Verify liquidator received collateral
    expect(await lpToken.balanceOf(liquidator.address)).to.be.gt(0);
  });
});
```

### Testing with External Services

Testing integrations with off-chain systems:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");
const axios = require('axios');
const sinon = require('sinon');

describe("Oracle Integration", function() {
  let dataConsumer;
  let owner;
  let mockServer;
  
  beforeEach(async function() {
    [owner] = await ethers.getSigners();
    
    // Setup mock for axios
    mockServer = sinon.stub(axios, 'get');
    
    // Mock successful price response
    mockServer.withArgs('https://api.pricing.example/v1/prices/ETH')
      .returns(Promise.resolve({
        data: {
          price: 2000.00,
          timestamp: Math.floor(Date.now() / 1000)
        }
      }));
    
    // Deploy oracle consumer contract
    const DataConsumer = await ethers.getContractFactory("PriceDataConsumer");
    dataConsumer = await DataConsumer.deploy();
  });
  
  afterEach(function() {
    // Restore axios to original state
    mockServer.restore();
  });
  
  it("fetches external price data for on-chain usage", async function() {
    // Request price update from off-chain source
    const tx = await dataConsumer.requestPriceUpdate("ETH");
    await tx.wait();
    
    // Wait for the off-chain processing (simulated)
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // The contract would normally receive this callback from an oracle node
    // Here we simulate it directly
    await dataConsumer.fulfillPriceRequest(
      ethers.utils.formatBytes32String("ETH"),
      ethers.utils.parseUnits("2000.00", 8),
      Math.floor(Date.now() / 1000)
    );
    
    // Verify the price was updated on-chain
    const priceData = await dataConsumer.getLatestPrice("ETH");
    expect(priceData.price).to.equal(ethers.utils.parseUnits("2000.00", 8));
    expect(priceData.timestamp).to.be.closeTo(Math.floor(Date.now() / 1000), 5);
  });
  
  it("handles failed API requests gracefully", async function() {
    // Mock a failed API response
    mockServer.withArgs('https://api.pricing.example/v1/prices/UNKNOWN')
      .returns(Promise.reject(new Error("Not found")));
    
    // Request price for an unknown symbol
    const tx = await dataConsumer.requestPriceUpdate("UNKNOWN");
    await tx.wait();
    
    // Wait for the off-chain processing (simulated)
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // The contract should handle failed requests
    // Here we simulate a response with a zero price and error flag
    await dataConsumer.fulfillPriceRequest(
      ethers.utils.formatBytes32String("UNKNOWN"),
      0,
      Math.floor(Date.now() / 1000),
      true // error flag
    );
    
    // Verify the contract handles the error correctly
    const priceData = await dataConsumer.getLatestPrice("UNKNOWN");
    expect(priceData.price).to.equal(0);
    expect(priceData.hasError).to.be.true;
  });
});
```

## Event Testing

### Event Emission Verification

Testing events emitted during contract interactions:

```javascript
describe("Event Testing", function() {
  let token;
  let exchange;
  let owner;
  let user;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    const Token = await ethers.getContractFactory("Token");
    token = await Token.deploy("Test Token", "TST");
    
    const Exchange = await ethers.getContractFactory("Exchange");
    exchange = await Exchange.deploy(token.address);
    
    // Mint and approve tokens
    await token.mint(owner.address, ethers.utils.parseEther("1000000"));
    await token.transfer(user.address, ethers.utils.parseEther("1000"));
    await token.connect(user).approve(exchange.address, ethers.utils.parseEther("1000"));
    
    // Add initial liquidity to exchange
    await token.approve(exchange.address, ethers.utils.parseEther("10000"));
    await exchange.addLiquidity(ethers.utils.parseEther("10000"), { value: ethers.utils.parseEther("10") });
  });
  
  it("emits Transfer events with correct parameters", async function() {
    // Watch for the Transfer event
    await expect(token.transfer(user.address, ethers.utils.parseEther("500")))
      .to.emit(token, 'Transfer')
      .withArgs(owner.address, user.address, ethers.utils.parseEther("500"));
  });
  
  it("emits multiple events in sequence during swap", async function() {
    // Perform a swap
    const swapTx = await exchange.connect(user).swapTokensForETH(
      ethers.utils.parseEther("100"),
      ethers.utils.parseEther("0.05") // Min ETH out
    );
    
    // Verify both events are emitted correctly
    await expect(swapTx)
      .to.emit(token, 'Transfer')
      .withArgs(user.address, exchange.address, ethers.utils.parseEther("100"));
    
    await expect(swapTx)
      .to.emit(exchange, 'TokenSwap')
      .withArgs(
        user.address,
        token.address,
        ethers.constants.AddressZero,
        ethers.utils.parseEther("100"),
        ethers.utils.anyValue // We don't know exact ETH amount but can check it's a valid value
      );
  });
  
  it("validates complex event structures", async function() {
    // Add an order to the exchange
    const placeTx = await exchange.connect(user).placeOrder(
      token.address,                     // Buy token
      ethers.utils.parseEther("100"),    // Buy amount
      { value: ethers.utils.parseEther("0.1") } // Sell ETH
    );
    
    // Get the event from transaction receipt
    const receipt = await placeTx.wait();
    const orderCreatedEvent = receipt.events?.find(e => e.event === 'OrderCreated');
    
    expect(orderCreatedEvent).to.not.be.undefined;
    expect(orderCreatedEvent.args).to.not.be.undefined;
    
    // Check specific event parameters
    const orderId = orderCreatedEvent.args.orderId;
    expect(orderId).to.not.be.undefined;
    
    // Retrieve order details and compare with event data
    const order = await exchange.getOrder(orderId);
    expect(order.maker).to.equal(user.address);
    expect(order.tokenBuy).to.equal(token.address);
    expect(order.amountBuy).to.equal(ethers.utils.parseEther("100"));
    expect(order.tokenSell).to.equal(ethers.constants.AddressZero); // ETH
    expect(order.amountSell).to.equal(ethers.utils.parseEther("0.1"));
  });
  
  it("filters events by indexed parameters", async function() {
    // Create multiple transfers
    await token.transfer(user.address, ethers.utils.parseEther("100"));
    
    const otherUser = ethers.Wallet.createRandom().connect(ethers.provider);
    await token.transfer(otherUser.address, ethers.utils.parseEther("200"));
    
    // Filter for transfers to specific user
    const filter = token.filters.Transfer(null, user.address);
    const events = await token.queryFilter(filter);
    
    // Should find 2 transfers: initial 1000 + 100
    expect(events.length).to.equal(2);
    expect(events[0].args.to).to.equal(user.address);
    expect(events[1].args.to).to.equal(user.address);
    
    // Total amount transferred should be 1000 + 100 = 1100
    const totalTransferred = events.reduce(
      (sum, event) => sum.add(event.args.value),
      ethers.BigNumber.from(0)
    );
    expect(totalTransferred).to.equal(ethers.utils.parseEther("1100"));
  });
});
```

### Event-Driven Workflows

Testing more complex event-based interactions:

```javascript
describe("Event-Driven Workflows", function() {
  let registry;
  let factory;
  let implementation;
  let owner;
  let user;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Deploy a proxy implementation
    const Implementation = await ethers.getContractFactory("TokenImplementation");
    implementation = await Implementation.deploy();
    
    // Deploy registry contract
    const Registry = await ethers.getContractFactory("ContractRegistry");
    registry = await Registry.deploy();
    
    // Deploy factory that creates proxies
    const Factory = await ethers.getContractFactory("TokenFactory");
    factory = await Factory.deploy(implementation.address, registry.address);
    
    // Give factory permission to register contracts
    await registry.grantRole(await registry.REGISTRAR_ROLE(), factory.address);
  });
  
  it("handles contract creation and registration workflow", async function() {
    // Watch for all events in the workflow
    const createTx = await factory.connect(user).createToken(
      "New Token",
      "NTK",
      ethers.utils.parseEther("1000000")
    );
    
    // Extract proxy address from event
    const receipt = await createTx.wait();
    const tokenCreatedEvent = receipt.events?.find(e => e.event === 'TokenCreated');
    expect(tokenCreatedEvent).to.not.be.undefined;
    
    const tokenAddress = tokenCreatedEvent.args.tokenAddress;
    expect(tokenAddress).to.be.properAddress;
    
    // Verify token was registered
    expect(await registry.isRegistered(tokenAddress)).to.be.true;
    expect(await registry.getContractType(tokenAddress)).to.equal("Token");
    
    // Check token contract works correctly
    const token = await ethers.getContractAt("TokenImplementation", tokenAddress);
    expect(await token.name()).to.equal("New Token");
    expect(await token.symbol()).to.equal("NTK");
    expect(await token.balanceOf(user.address)).to.equal(ethers.utils.parseEther("1000000"));
  });
  
  it("creates a subscription notification system", async function() {
    // Create a new token
    const createTx = await factory.connect(user).createToken(
      "Sub Token", 
      "SUB",
      ethers.utils.parseEther("1000000")
    );
    
    const receipt = await createTx.wait();
    const tokenAddress = receipt.events?.find(e => e.event === 'TokenCreated').args.tokenAddress;
    const token = await ethers.getContractAt("TokenImplementation", tokenAddress);
    
    // Deploy notification system that listens for Transfer events
    const Notifier = await ethers.getContractFactory("TransferNotifier");
    const notifier = await Notifier.deploy();
    
    // Subscribe to large transfers
    await notifier.subscribeToLargeTransfers(
      token.address,
      ethers.utils.parseEther("10000")
    );
    
    // Execute a large transfer that should trigger notification
    const recipient = ethers.Wallet.createRandom().address;
    await token.connect(user).transfer(recipient, ethers.utils.parseEther("50000"));
    
    // Check if notification was recorded
    const notifications = await notifier.getNotifications();
    expect(notifications.length).to.equal(1);
    expect(notifications[0].token).to.equal(token.address);
    expect(notifications[0].from).to.equal(user.address);
    expect(notifications[0].to).to.equal(recipient);
    expect(notifications[0].amount).to.equal(ethers.utils.parseEther("50000"));
    
    // Small transfer should not trigger notification
    await token.connect(user).transfer(recipient, ethers.utils.parseEther("100"));
    expect(await notifier.getNotifications()).to.have.length(1); // Still just one
  });
});
```

## Gas Optimization Verification

### Measuring Gas Usage

Testing gas efficiency of contract operations:

```javascript
describe("Gas Usage Analysis", function() {
  let gasEfficient;
  let gasInefficient;
  let owner;
  let user;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Deploy gas efficient version
    const GasEfficient = await ethers.getContractFactory("GasEfficientToken");
    gasEfficient = await GasEfficient.deploy("Efficient", "EFF");
    
    // Deploy gas inefficient version
    const GasInefficient = await ethers.getContractFactory("GasInefficientToken");
    gasInefficient = await GasInefficient.deploy("Inefficient", "INEFF");
    
    // Setup test data
    await gasEfficient.mint(owner.address, ethers.utils.parseEther("1000000"));
    await gasInefficient.mint(owner.address, ethers.utils.parseEther("1000000"));
  });
  
  it("compares gas usage between implementations", async function() {
    // Transfer with efficient implementation
    const efficientTx = await gasEfficient.transfer(
      user.address,
      ethers.utils.parseEther("1000")
    );
    const efficientReceipt = await efficientTx.wait();
    const efficientGasUsed = efficientReceipt.gasUsed;
    
    // Transfer with inefficient implementation
    const inefficientTx = await gasInefficient.transfer(
      user.address,
      ethers.utils.parseEther("1000")
    );
    const inefficientReceipt = await inefficientTx.wait();
    const inefficientGasUsed = inefficientReceipt.gasUsed;
    
    console.log(`Gas used (efficient): ${efficientGasUsed.toString()}`);
    console.log(`Gas used (inefficient): ${inefficientGasUsed.toString()}`);
    console.log(`Savings: ${inefficientGasUsed.sub(efficientGasUsed).toString()}`);
    
    // Efficient implementation should use less gas
    expect(efficientGasUsed).to.be.lt(inefficientGasUsed);
  });
  
  it("verifies batch operations save gas compared to individual calls", async function() {
    // Prepare recipient addresses
    const recipients = [];
    const amounts = [];
    for (let i = 0; i < 10; i++) {
      recipients.push(ethers.Wallet.createRandom().address);
      amounts.push(ethers.utils.parseEther("100"));
    }
    
    // Measure gas for individual transfers
    let totalGasIndividual = ethers.BigNumber.from(0);
    for (let i = 0; i < recipients.length; i++) {
      const tx = await gasEfficient.transfer(recipients[i], amounts[i]);
      const receipt = await tx.wait();
      totalGasIndividual = totalGasIndividual.add(receipt.gasUsed);
    }
    
    // Reset balances for batch test
    await gasEfficient.transferFrom(
      owner.address, 
      ethers.Wallet.createRandom().address,
      await gasEfficient.balanceOf(owner.address)
    );
    await gasEfficient.mint(owner.address, ethers.utils.parseEther("1000000"));
    
    // Measure gas for batch transfer
    const batchTx = await gasEfficient.batchTransfer(recipients, amounts);
    const batchReceipt = await batchTx.wait();
    const batchGasUsed = batchReceipt.gasUsed;
    
    console.log(`Gas used (10 individual transfers): ${totalGasIndividual.toString()}`);
    console.log(`Gas used (batch transfer of 10): ${batchGasUsed.toString()}`);
    console.log(`Savings: ${totalGasIndividual.sub(batchGasUsed).toString()}`);
    
    // Batch should be more efficient
    expect(batchGasUsed).to.be.lt(totalGasIndividual);
  });
  
  it("verifies storage optimization techniques", async function() {
    // Deploy contracts with different storage patterns
    const NormalStorage = await ethers.getContractFactory("NormalStorageContract");
    const normalStorage = await NormalStorage.deploy();
    
    const PackedStorage = await ethers.getContractFactory("PackedStorageContract");
    const packedStorage = await PackedStorage.deploy();
    
    // Test writing multiple values
    const normalTx = await normalStorage.setValues(42, 100, true);
    const normalReceipt = await normalTx.wait();
    
    const packedTx = await packedStorage.setValues(42, 100, true);
    const packedReceipt = await packedTx.wait();
    
    console.log(`Gas used (normal storage): ${normalReceipt.gasUsed.toString()}`);
    console.log(`Gas used (packed storage): ${packedReceipt.gasUsed.toString()}`);
    
    // Packed storage should use less gas
    expect(packedReceipt.gasUsed).to.be.lt(normalReceipt.gasUsed);
    
    // Verify values are stored correctly despite packing
    expect(await packedStorage.getValue1()).to.equal(42);
    expect(await packedStorage.getValue2()).to.equal(100);
    expect(await packedStorage.getFlag()).to.equal(true);
  });
});
```

### Optimizing Contract Interactions

Testing composite operations for gas efficiency:

```javascript
describe("Optimized Contract Interactions", function() {
  let token;
  let vault;
  let owner;
  let user;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    const Token = await ethers.getContractFactory("Token");
    token = await Token.deploy("Test Token", "TST");
    await token.mint(owner.address, ethers.utils.parseEther("1000000"));
    await token.transfer(user.address, ethers.utils.parseEther("10000"));
    
    const Vault = await ethers.getContractFactory("TokenVault");
    vault = await Vault.deploy(token.address);
  });
  
  it("optimizes deposit interactions", async function() {
    // Option 1: User approves, then vault pulls tokens
    await token.connect(user).approve(vault.address, ethers.utils.parseEther("1000"));
    const separateTx = await vault.connect(user).deposit(ethers.utils.parseEther("1000"));
    const separateReceipt = await separateTx.wait();
    
    // Reset for next test
    await vault.connect(user).withdraw(ethers.utils.parseEther("1000"));
    
    // Option 2: User transfers tokens and deposits in one call
    await token.connect(user).approve(vault.address, ethers.utils.parseEther("1000"));
    const combinedTx = await vault.connect(user).depositWithTransfer(ethers.utils.parseEther("1000"));
    const combinedReceipt = await combinedTx.wait();
    
    console.log(`Gas used (separate approve+deposit): ${separateReceipt.gasUsed.toString()}`);
    console.log(`Gas used (combined transfer+deposit): ${combinedReceipt.gasUsed.toString()}`);
    
    // Combined operation should be more efficient
    expect(combinedReceipt.gasUsed).to.be.lt(separateReceipt.gasUsed);
    
    // Both approaches should result in the same vault balance
    expect(await vault.balanceOf(user.address)).to.equal(ethers.utils.parseEther("1000"));
  });
  
  it("tests read vs. write gas efficiency", async function() {
    // Setup data
    for (let i = 0; i < 10; i++) {
      await vault.addTrustedContract(ethers.Wallet.createRandom().address);
    }
    
    // Option 1: Read all data and process off-chain
    const readTx = await vault.getAllTrustedContracts();
    const readReceipt = await readTx.wait();
    
    // Option 2: Process on-chain
    const processTx = await vault.countValidTrustedContracts();
    const processReceipt = await processTx.wait();
    
    console.log(`Gas used (read data): ${readReceipt.gasUsed.toString()}`);
    console.log(`Gas used (process on-chain): ${processReceipt.gasUsed.toString()}`);
    
    // Reading should be more gas efficient than processing on-chain
    expect(readReceipt.gasUsed).to.be.lt(processReceipt.gasUsed);
  });
});
```

## Integration with External Systems

### Testing Database Interactions

Testing blockchain-to-database flows:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");
const { MongoClient } = require('mongodb');
const sinon = require('sinon');

describe("Database Integration", function() {
  let chainBridge;
  let owner;
  let user;
  let mongoClientStub;
  let dbStub;
  let collectionsStub;
  
  beforeEach(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Setup MongoDB stubs
    collectionsStub = {
      transactions: {
        insertOne: sinon.stub().resolves({ acknowledged: true, insertedId: 'mockId' }),
        findOne: sinon.stub().resolves(null),
        updateOne: sinon.stub().resolves({ acknowledged: true, modifiedCount: 1 })
      }
    };
    
    dbStub = {
      collection: sinon.stub().callsFake(name => collectionsStub[name])
    };
    
    mongoClientStub = {
      connect: sinon.stub().resolves(),
      db: sinon.stub().returns(dbStub),
      close: sinon.stub().resolves()
    };
    
    // Mock the MongoDB client constructor
    sinon.stub(MongoClient, 'connect').resolves(mongoClientStub);
    
    // Deploy chain-to-db bridge contract
    const ChainBridge = await ethers.getContractFactory("BlockchainDBBridge");
    chainBridge = await ChainBridge.deploy();
  });
  
  afterEach(function() {
    // Restore stubs
    sinon.restore();
  });
  
  it("records blockchain events to database", async function() {
    // Create an event on the blockchain
    const tx = await chainBridge.connect(user).recordTransaction(
      "Product Purchase",
      ethers.utils.parseEther("100"),
      ethers.utils.formatBytes32String("SKU123")
    );
    const receipt = await tx.wait();
    
    // Extract transaction ID from event
    const event = receipt.events.find(e => e.event === 'TransactionRecorded');
    const txId = event.args.transactionId;
    
    // Simulate off-chain worker processing the event
    const handler = require('../scripts/event-handler');
    await handler.processEvent(event, "mongodb://localhost:27017/testdb");
    
    // Verify data was written to MongoDB
    expect(collectionsStub.transactions.insertOne.called).to.be.true;
    
    const document = collectionsStub.transactions.insertOne.firstCall.args[0];
    expect(document).to.have.property('transactionId', txId.toString());
    expect(document).to.have.property('type', 'Product Purchase');
    expect(document).to.have.property('amount');
    expect(document).to.have.property('reference', ethers.utils.parseBytes32String("SKU123"));
    expect(document).to.have.property('userAddress', user.address.toLowerCase());
    expect(document).to.have.property('timestamp').that.is.a('number');
    expect(document).to.have.property('blockNumber', receipt.blockNumber);
    expect(document).to.have.property('status', 'confirmed');
  });
  
  it("syncs database updates back to blockchain", async function() {
    // Setup a transaction ID in the contract
    const tx = await chainBridge.connect(user).recordTransaction(
      "Account Credit",
      ethers.utils.parseEther("200"),
      ethers.utils.formatBytes32String("CREDIT001")
    );
    const receipt = await tx.wait();
    const event = receipt.events.find(e => e.event === 'TransactionRecorded');
    const txId = event.args.transactionId;
    
    // Simulate database processing and status update
    collectionsStub.transactions.findOne.resolves({
      transactionId: txId.toString(),
      status: 'processed',
      processedAt: new Date().toISOString(),
      extReference: 'EXT123'
    });
    
    // Run sync operation (would normally be triggered by a cron job)
    const syncScript = require('../scripts/sync-db-to-chain');
    await syncScript.syncPendingTransactions("mongodb://localhost:27017/testdb");
    
    // Verify blockchain was updated with DB status
    const txInfo = await chainBridge.getTransaction(txId);
    expect(txInfo.status).to.equal(2); // 'Processed' status code
    expect(txInfo.externalReference).to.equal('EXT123');
    
    // Verify DB was updated to reflect sync
    expect(collectionsStub.transactions.updateOne.called).to.be.true;
    const updateQuery = collectionsStub.transactions.updateOne.firstCall.args[0];
    const updateValues = collectionsStub.transactions.updateOne.firstCall.args[1];
    expect(updateQuery).to.deep.equal({ transactionId: txId.toString() });
    expect(updateValues).to.have.property('$set');
    expect(updateValues.$set).to.have.property('synced', true);
  });
});
```

### Testing API Integration

Testing blockchain-to-API flows:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");
const nock = require('nock');
const axios = require('axios');

describe("API Integration Tests", function() {
  let apiConsumer;
  let owner;
  let mockApi;
  
  beforeEach(async function() {
    [owner] = await ethers.getSigners();
    
    // Deploy contract that interacts with APIs
    const ApiConsumer = await ethers.getContractFactory("ApiConsumer");
    apiConsumer = await ApiConsumer.deploy();
    
    // Setup mock API endpoints
    mockApi = nock('https://api.example.com');
    
    // Mock successful response
    mockApi.get('/data/latest')
      .reply(200, {
        value: 123456,
        timestamp: Math.floor(Date.now() / 1000)
      });
    
    // Mock error response
    mockApi.get('/data/error')
      .reply(500, {
        error: "Internal server error"
      });
  });
  
  afterEach(function() {
    nock.cleanAll();
  });
  
  it("processes API data via off-chain relay", async function() {
    // On-chain request for off-chain data
    const tx = await apiConsumer.requestData("LATEST_VALUE");
    const receipt = await tx.wait();
    
    // Extract request ID from event
    const event = receipt.events?.find(e => e.event === 'DataRequested');
    const requestId = event.args.requestId;
    
    // Simulate off-chain relay fetching the data
    const response = await axios.get('https://api.example.com/data/latest');
    
    // Submit data back on-chain
    await apiConsumer.fulfillRequest(
      requestId,
      response.data.value,
      response.data.timestamp
    );
    
    // Verify data is accessible on-chain
    const retrievedData = await apiConsumer.getData("LATEST_VALUE");
    expect(retrievedData.value).to.equal(123456);
    expect(retrievedData.timestamp).to.be.closeTo(Math.floor(Date.now() / 1000), 10);
    expect(retrievedData.exists).to.be.true;
  });
  
  it("handles API failures gracefully", async function() {
    // Request data that will result in API error
    const tx = await apiConsumer.requestData("ERROR_VALUE");
    const receipt = await tx.wait();
    const requestId = receipt.events?.find(e => e.event === 'DataRequested').args.requestId;
    
    // Simulate relay attempting to fetch data and encountering error
    let errorOccurred = false;
    try {
      await axios.get('https://api.example.com/data/error');
    } catch (error) {
      errorOccurred = true;
      
      // Report failure back to chain
      await apiConsumer.reportRequestError(
        requestId,
        "API server error"
      );
    }
    
    expect(errorOccurred).to.be.true;
    
    // Check that error was recorded on-chain
    const requestStatus = await apiConsumer.getRequestStatus(requestId);
    expect(requestStatus.fulfilled).to.be.false;
    expect(requestStatus.errorMessage).to.equal("API server error");
  });
  
  it("respects rate limiting in API integrations", async function() {
    // Setup rate limiting mocks
    let requestCount = 0;
    mockApi.get('/rate-limited')
      .times(5) // Allow 5 requests
      .reply(200, () => {
        requestCount++;
        return { success: true, count: requestCount };
      });
    
    mockApi.get('/rate-limited')
      .reply(429, { error: "Too many requests" }); // Rate limit after 5
    
    // Create multiple data requests
    const requestIds = [];
    for (let i = 0; i < 6; i++) {
      const tx = await apiConsumer.requestData(`RATE_TEST_${i}`);
      const receipt = await tx.wait();
      const requestId = receipt.events?.find(e => e.event === 'DataRequested').args.requestId;
      requestIds.push(requestId);
    }
    
    // Simulate off-chain relay processing requests with rate limiting
    const results = [];
    for (let i = 0; i < requestIds.length; i++) {
      try {
        const response = await axios.get('https://api.example.com/rate-limited');
        results.push({ success: true, data: response.data });
        
        // Report success to chain
        await apiConsumer.fulfillRequest(
          requestIds[i],
          response.data.count,
          Math.floor(Date.now() / 1000)
        );
      } catch (error) {
        results.push({ success: false, error: error.response?.data || error.message });
        
        // Report error to chain
        await apiConsumer.reportRequestError(
          requestIds[i],
          "Rate limit exceeded"
        );
      }
    }
    
    // Verify results
    expect(results.filter(r => r.success)).to.have.length(5); // 5 successful
    expect(results.filter(r => !r.success)).to.have.length(1); // 1 rate limited
    
    // Check on-chain status
    for (let i = 0; i < 5; i++) {
      const data = await apiConsumer.getData(`RATE_TEST_${i}`);
      expect(data.exists).to.be.true;
    }
    
    const lastRequestStatus = await apiConsumer.getRequestStatus(requestIds[5]);
    expect(lastRequestStatus.errorMessage).to.equal("Rate limit exceeded");
  });
});
```

## Conclusion

Integration testing is a vital part of blockchain application verification. By testing how components interact and behave as a system, developers can identify issues that wouldn't be apparent from unit tests alone, such as contract interaction bugs, event handling problems, or gas inefficiencies.

For ProzChain applications, effective integration testing should include:
- Testing direct contract-to-contract interactions
- Validating multi-contract workflows
- Verifying event emissions and handling
- Measuring and optimizing gas usage
- Testing integrations with external systems

These tests provide confidence that the application will work correctly in real-world scenarios, with components interacting properly across the system. In the next chapter, we'll explore end-to-end testing to validate complete user workflows and journeys.

