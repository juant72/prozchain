# Unit Testing

## Overview

Unit testing forms the foundation of the testing pyramid for blockchain applications. It involves testing individual components in isolation to verify their correctness. For ProzChain applications, this typically means testing individual functions within smart contracts, verifying state transitions, and ensuring proper error handling.

This chapter covers techniques for writing effective unit tests for ProzChain smart contracts, including how to structure tests, mock dependencies, and verify contract behavior.

## Component-Level Testing Strategy

### Testing Smart Contract Units

Smart contracts should be tested at the component level, with each function tested in isolation:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Token Contract", function() {
  let Token;
  let token;
  let owner;
  let addr1;
  let addr2;
  
  beforeEach(async function() {
    // Get the ContractFactory and signers
    Token = await ethers.getContractFactory("Token");
    [owner, addr1, addr2] = await ethers.getSigners();
    
    // Deploy a new Token contract for each test
    token = await Token.deploy("Test Token", "TST", 1000000);
    await token.deployed();
  });
  
  // Test individual functions
  describe("Deployment", function() {
    it("should set the correct token name and symbol", async function() {
      expect(await token.name()).to.equal("Test Token");
      expect(await token.symbol()).to.equal("TST");
    });
    
    it("should assign the total supply to the owner", async function() {
      const ownerBalance = await token.balanceOf(owner.address);
      expect(await token.totalSupply()).to.equal(ownerBalance);
    });
  });
  
  describe("Transactions", function() {
    it("should transfer tokens between accounts", async function() {
      // Transfer from owner to addr1
      await token.transfer(addr1.address, 50);
      const addr1Balance = await token.balanceOf(addr1.address);
      expect(addr1Balance).to.equal(50);
      
      // Transfer from addr1 to addr2
      await token.connect(addr1).transfer(addr2.address, 25);
      const addr2Balance = await token.balanceOf(addr2.address);
      expect(addr2Balance).to.equal(25);
    });
    
    it("should fail if sender doesn't have enough tokens", async function() {
      const initialOwnerBalance = await token.balanceOf(owner.address);
      
      // Try to send more tokens than owner has
      await expect(
        token.connect(addr1).transfer(owner.address, 1)
      ).to.be.revertedWith("ERC20: transfer amount exceeds balance");
      
      // Owner balance shouldn't have changed
      expect(await token.balanceOf(owner.address)).to.equal(initialOwnerBalance);
    });
  });
});
```

### Test Organization Principles

Organize tests for maximum clarity and maintainability:

1. **Hierarchical Structure**:
   - Group tests by contract/component
   - Subgroup by function/feature
   - Further subgroup by scenario/case

2. **Naming Conventions**:
   - Use descriptive file names: `TokenTransfer.test.js`
   - Use descriptive test names: `"reverts when transfer amount exceeds balance"`

3. **Independent Tests**:
   - Each test should be self-contained
   - Tests should not rely on state changes from other tests

Example of structured test organization:

```javascript
// test/unit/token/TokenAllowance.test.js
describe("Token", function() {
  // Shared setup code...
  
  describe("allowance functionality", function() {
    describe("approve", function() {
      it("sets correct allowance value", async function() {
        // Test implementation...
      });
      
      it("emits Approval event with correct parameters", async function() {
        // Test implementation...
      });
      
      it("replaces previous allowance", async function() {
        // Test implementation...
      });
    });
    
    describe("transferFrom", function() {
      beforeEach(async function() {
        // Setup for transferFrom tests...
      });
      
      it("transfers tokens when sender has sufficient allowance", async function() {
        // Test implementation...
      });
      
      it("reduces allowance after successful transfer", async function() {
        // Test implementation...
      });
      
      it("reverts when sender has insufficient allowance", async function() {
        // Test implementation...
      });
    });
  });
});
```

## Mocking Dependencies

### Contract Dependencies

Creating mock contracts to isolate the component under test:

```solidity
// contracts/mocks/MockPriceOracle.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

contract MockPriceOracle {
    mapping(address => uint256) public prices;
    
    function setPrice(address asset, uint256 price) external {
        prices[asset] = price;
    }
    
    function getPrice(address asset) external view returns (uint256) {
        return prices[asset];
    }
}
```

Using mock contracts in tests:

```javascript
// test/unit/lending/InterestCalculation.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("LendingPool interest calculation", function() {
  let LendingPool;
  let lendingPool;
  let mockPriceOracle;
  let mockToken;
  
  beforeEach(async function() {
    // Deploy mock dependencies
    const MockPriceOracle = await ethers.getContractFactory("MockPriceOracle");
    mockPriceOracle = await MockPriceOracle.deploy();
    
    const MockToken = await ethers.getContractFactory("MockToken");
    mockToken = await MockToken.deploy("Mock Token", "MTK", 1000000);
    
    // Deploy contract under test with mock dependencies
    LendingPool = await ethers.getContractFactory("LendingPool");
    lendingPool = await LendingPool.deploy(mockPriceOracle.address);
    
    // Configure mocks
    await mockPriceOracle.setPrice(mockToken.address, ethers.utils.parseEther("100"));
  });
  
  it("calculates interest correctly", async function() {
    // Test using mock dependencies
    await lendingPool.setInterestRate(mockToken.address, 1000); // 10% APR
    
    // Test interest calculation
    const interest = await lendingPool.calculateInterest(
      mockToken.address,
      ethers.utils.parseEther("1000"),
      30 // days
    );
    
    // 1000 * 10% * 30/365 = 8.219... ETH
    expect(interest).to.be.closeTo(
      ethers.utils.parseEther("8.219"),
      ethers.utils.parseEther("0.001")
    );
  });
});
```

### Function Mocks

Using function mocking in Hardhat:

```javascript
// test/unit/governance/Proposal.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ProposalExecutor", function() {
  let proposalExecutor;
  let mockRegistry;
  
  beforeEach(async function() {
    const Registry = await ethers.getContractFactory("Registry");
    mockRegistry = await Registry.deploy();
    
    // Deploy contract under test
    const ProposalExecutor = await ethers.getContractFactory("ProposalExecutor");
    proposalExecutor = await ProposalExecutor.deploy(mockRegistry.address);
    
    // Mock the hasPermission function in Registry
    await ethers.provider.send("hardhat_setCode", [
      mockRegistry.address,
      mockRegistry.interface.encodeFunctionData("hasPermission", [
        ethers.constants.AddressZero,
        ethers.utils.id("EXECUTE_ROLE")
      ])
    ]);
  });
  
  it("executes proposals when sender has permission", async function() {
    // Test with mocked permission check
    // ...
  });
});
```

### External System Simulation

Creating simulations of external systems:

```javascript
// test/helpers/simulateExternalSystems.js
const { ethers } = require("hardhat");

/**
 * Simulates an external oracle update
 * @param {Contract} oracle - Oracle contract
 * @param {string} pair - Trading pair
 * @param {BigNumber} price - New price
 * @param {object} options - Simulation options
 */
async function simulateOracleUpdate(oracle, pair, price, options = {}) {
  // Get the oracle keeper's private key
  const keeperPrivateKey = process.env.ORACLE_KEEPER_KEY || ethers.Wallet.createRandom().privateKey;
  const keeper = new ethers.Wallet(keeperPrivateKey, ethers.provider);
  
  // Fund the keeper if needed
  if ((await ethers.provider.getBalance(keeper.address)).eq(0)) {
    const [deployer] = await ethers.getSigners();
    await deployer.sendTransaction({
      to: keeper.address,
      value: ethers.utils.parseEther("1")
    });
  }
  
  // Calculate timestamp
  const timestamp = options.timestamp || Math.floor(Date.now() / 1000);
  
  // Prepare and sign the price update
  const messageHash = ethers.utils.solidityKeccak256(
    ["string", "uint256", "uint256"],
    [pair, price, timestamp]
  );
  
  const messageHashBinary = ethers.utils.arrayify(messageHash);
  const signature = await keeper.signMessage(messageHashBinary);
  
  // Update the oracle
  await oracle.updatePrice(pair, price, timestamp, signature);
  
  return { keeper, signature };
}

module.exports = {
  simulateOracleUpdate
};
```

## Testing State Transitions

### Initial State Verification

Verifying correct initial state:

```javascript
// test/unit/staking/StakingPool.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("StakingPool", function() {
  let StakingPool;
  let stakingPool;
  let rewardToken;
  let stakingToken;
  let owner;
  
  beforeEach(async function() {
    [owner, user1, user2] = await ethers.getSigners();
    
    // Deploy tokens
    const Token = await ethers.getContractFactory("Token");
    stakingToken = await Token.deploy("Staking Token", "STK", 1000000);
    rewardToken = await Token.deploy("Reward Token", "RWD", 1000000);
    
    // Deploy staking pool
    StakingPool = await ethers.getContractFactory("StakingPool");
    stakingPool = await StakingPool.deploy(
      stakingToken.address,
      rewardToken.address
    );
    
    // Fund the staking pool with reward tokens
    await rewardToken.transfer(stakingPool.address, ethers.utils.parseEther("1000"));
  });
  
  describe("Initial state", function() {
    it("has correct staking token address", async function() {
      expect(await stakingPool.stakingToken()).to.equal(stakingToken.address);
    });
    
    it("has correct reward token address", async function() {
      expect(await stakingPool.rewardToken()).to.equal(rewardToken.address);
    });
    
    it("has zero total staked amount", async function() {
      expect(await stakingPool.totalStaked()).to.equal(0);
    });
    
    it("has expected reward token balance", async function() {
      const balance = await rewardToken.balanceOf(stakingPool.address);
      expect(balance).to.equal(ethers.utils.parseEther("1000"));
    });
  });
  
  // Additional test suites...
});
```

### State Transition Testing

Verifying correct state changes:

```javascript
// test/unit/staking/StakingTransitions.test.js
describe("StakingPool state transitions", function() {
  // Setup code...
  
  describe("staking", function() {
    it("updates user staked balance", async function() {
      // Preconditions
      const stakeAmount = ethers.utils.parseEther("100");
      await stakingToken.transfer(user1.address, stakeAmount);
      await stakingToken.connect(user1).approve(stakingPool.address, stakeAmount);
      
      // Initial state check
      expect(await stakingPool.balanceOf(user1.address)).to.equal(0);
      
      // Action
      await stakingPool.connect(user1).stake(stakeAmount);
      
      // Post-state check
      expect(await stakingPool.balanceOf(user1.address)).to.equal(stakeAmount);
    });
    
    it("transfers tokens from user to staking pool", async function() {
      // Preconditions
      const stakeAmount = ethers.utils.parseEther("100");
      await stakingToken.transfer(user1.address, stakeAmount);
      await stakingToken.connect(user1).approve(stakingPool.address, stakeAmount);
      
      // Initial state check
      const initialUserBalance = await stakingToken.balanceOf(user1.address);
      const initialPoolBalance = await stakingToken.balanceOf(stakingPool.address);
      
      // Action
      await stakingPool.connect(user1).stake(stakeAmount);
      
      // Post-state check
      expect(await stakingToken.balanceOf(user1.address)).to.equal(initialUserBalance.sub(stakeAmount));
      expect(await stakingToken.balanceOf(stakingPool.address)).to.equal(initialPoolBalance.add(stakeAmount));
    });
    
    it("increases total staked amount", async function() {
      // Preconditions
      const stakeAmount = ethers.utils.parseEther("100");
      await stakingToken.transfer(user1.address, stakeAmount);
      await stakingToken.connect(user1).approve(stakingPool.address, stakeAmount);
      
      // Initial state check
      const initialTotalStaked = await stakingPool.totalStaked();
      
      // Action
      await stakingPool.connect(user1).stake(stakeAmount);
      
      // Post-state check
      expect(await stakingPool.totalStaked()).to.equal(initialTotalStaked.add(stakeAmount));
    });
  });
});
```

### Invariant Checking

Testing properties that should always hold true:

```javascript
// test/unit/token/TokenInvariants.test.js
describe("Token invariants", function() {
  // Setup code...
  
  it("total supply equals sum of all balances", async function() {
    // Initial check
    const totalSupply = await token.totalSupply();
    let sumOfBalances = ethers.BigNumber.from(0);
    
    for (const account of [owner, user1, user2]) {
      sumOfBalances = sumOfBalances.add(await token.balanceOf(account.address));
    }
    
    expect(sumOfBalances).to.equal(totalSupply);
    
    // Transfer some tokens
    await token.transfer(user1.address, 100);
    await token.connect(user1).transfer(user2.address, 50);
    
    // Check after transfers
    let newSumOfBalances = ethers.BigNumber.from(0);
    for (const account of [owner, user1, user2]) {
      newSumOfBalances = newSumOfBalances.add(await token.balanceOf(account.address));
    }
    
    // Invariant should still hold
    expect(newSumOfBalances).to.equal(totalSupply);
  });
});
```

## Testing Cryptographic Operations

### Testing Signature Verification

Verifying signature validation logic:

```javascript
// test/unit/security/SignatureVerification.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("SignatureVerifier", function() {
  let verifier;
  let owner;
  let signer;
  let attacker;
  
  beforeEach(async function() {
    [owner, signer, attacker] = await ethers.getSigners();
    
    const SignatureVerifier = await ethers.getContractFactory("SignatureVerifier");
    verifier = await SignatureVerifier.deploy();
    
    // Register authorized signer
    await verifier.addSigner(signer.address);
  });
  
  describe("verification", function() {
    it("accepts valid signatures from authorized signer", async function() {
      // Prepare message
      const message = "Test message";
      const messageHash = ethers.utils.id(message);
      const messageHashBytes = ethers.utils.arrayify(messageHash);
      
      // Sign the message
      const signature = await signer.signMessage(messageHashBytes);
      
      // Verify
      expect(await verifier.verify(message, signature)).to.be.true;
    });
    
    it("rejects signatures from unauthorized signers", async function() {
      // Prepare message
      const message = "Test message";
      const messageHash = ethers.utils.id(message);
      const messageHashBytes = ethers.utils.arrayify(messageHash);
      
      // Sign with unauthorized account
      const signature = await attacker.signMessage(messageHashBytes);
      
      // Verify
      expect(await verifier.verify(message, signature)).to.be.false;
    });
    
    it("rejects tampered messages", async function() {
      // Prepare message
      const message = "Test message";
      const messageHash = ethers.utils.id(message);
      const messageHashBytes = ethers.utils.arrayify(messageHash);
      
      // Sign the message
      const signature = await signer.signMessage(messageHashBytes);
      
      // Verify with tampered message
      const tamperedMessage = "Tampered message";
      expect(await verifier.verify(tamperedMessage, signature)).to.be.false;
    });
  });
});
```

### Testing Hashing Functions

Validating hashing operations:

```javascript
// test/unit/security/HashingFunctions.test.js
describe("MerkleTree", function() {
  let merkleTree;
  
  beforeEach(async function() {
    const MerkleTree = await ethers.getContractFactory("MerkleTree");
    merkleTree = await MerkleTree.deploy();
  });
  
  describe("leaf hashing", function() {
    it("computes leaf hash correctly", async function() {
      const data = ethers.utils.toUtf8Bytes("leaf data");
      
      // Calculate expected hash (prefix 0x00 for leaf nodes)
      const expectedHash = ethers.utils.keccak256(
        ethers.utils.concat([
          ethers.utils.hexZeroPad("0x00", 1),
          data
        ])
      );
      
      // Compare with contract's hash calculation
      const actualHash = await merkleTree.hashLeaf(data);
      expect(actualHash).to.equal(expectedHash);
    });
  });
  
  describe("node hashing", function() {
    it("computes node hash correctly", async function() {
      const left = ethers.utils.id("left child");
      const right = ethers.utils.id("right child");
      
      // Calculate expected hash (prefix 0x01 for internal nodes)
      const expectedHash = ethers.utils.keccak256(
        ethers.utils.concat([
          ethers.utils.hexZeroPad("0x01", 1),
          left,
          right
        ])
      );
      
      // Compare with contract's hash calculation
      const actualHash = await merkleTree.hashNode(left, right);
      expect(actualHash).to.equal(expectedHash);
    });
    
    it("sorts child hashes before hashing", async function() {
      // Choose hashes such that left > right
      const left = ethers.utils.hexZeroPad("0xff", 32);
      const right = ethers.utils.hexZeroPad("0x01", 32);
      
      // Contract should sort, so hash(left, right) == hash(right, left)
      const hash1 = await merkleTree.hashNode(left, right);
      const hash2 = await merkleTree.hashNode(right, left);
      
      expect(hash1).to.equal(hash2);
    });
  });
});
```

### Testing Encryption and Decryption

Validating encryption functions:

```javascript
// test/unit/security/Encryption.test.js
describe("EncryptionUtils", function() {
  let encryptionUtils;
  
  beforeEach(async function() {
    const EncryptionUtils = await ethers.getContractFactory("EncryptionUtils");
    encryptionUtils = await EncryptionUtils.deploy();
  });
  
  describe("symmetric encryption", function() {
    it("encrypted data can be decrypted", async function() {
      const data = ethers.utils.toUtf8Bytes("secret message");
      const key = ethers.utils.id("encryption key").slice(0, 34); // 16 bytes + 0x
      
      // Encrypt
      const encrypted = await encryptionUtils.encrypt(data, key);
      
      // Decrypt
      const decrypted = await encryptionUtils.decrypt(encrypted, key);
      
      // Compare
      expect(ethers.utils.toUtf8String(decrypted)).to.equal("secret message");
    });
    
    it("cannot decrypt with wrong key", async function() {
      const data = ethers.utils.toUtf8Bytes("secret message");
      const key = ethers.utils.id("correct key").slice(0, 34);
      const wrongKey = ethers.utils.id("wrong key").slice(0, 34);
      
      // Encrypt
      const encrypted = await encryptionUtils.encrypt(data, key);
      
      // Decrypt with wrong key
      const decrypted = await encryptionUtils.decrypt(encrypted, wrongKey);
      
      // Should not match original
      expect(ethers.utils.toUtf8String(decrypted)).to.not.equal("secret message");
    });
  });
});
```

## Advanced Unit Testing Techniques

### Snapshot Testing

Using Hardhat snapshots to reset state:

```javascript
// test/unit/advanced/SnapshotTesting.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Using snapshots", function() {
  let token;
  let owner;
  let user1;
  let snapshotId;
  
  before(async function() {
    [owner, user1] = await ethers.getSigners();
    
    const Token = await ethers.getContractFactory("Token");
    token = await Token.deploy("Snapshot Test", "SNAP", 1000000);
  });
  
  beforeEach(async function() {
    // Take a snapshot before each test
    snapshotId = await ethers.provider.send("evm_snapshot", []);
  });
  
  afterEach(async function() {
    // Revert to the snapshot after each test
    await ethers.provider.send("evm_revert", [snapshotId]);
  });
  
  it("first test mutates state", async function() {
    // Initial state
    const initialBalance = await token.balanceOf(owner.address);
    
    // Mutate state
    await token.transfer(user1.address, 1000);
    
    // Verify mutation
    expect(await token.balanceOf(user1.address)).to.equal(1000);
  });
  
  it("second test starts with fresh state", async function() {
    // State should be reset - user1 has no tokens
    expect(await token.balanceOf(user1.address)).to.equal(0);
    
    // Owner should have full balance
    expect(await token.balanceOf(owner.address)).to.equal(1000000);
  });
});
```

### Fuzz Testing

Basic fuzz testing with random inputs:

```javascript
// test/unit/advanced/FuzzTesting.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Calculator fuzz testing", function() {
  let calculator;
  
  before(async function() {
    const Calculator = await ethers.getContractFactory("Calculator");
    calculator = await Calculator.deploy();
  });
  
  it("add function is commutative", async function() {
    for (let i = 0; i < 100; i++) {
      // Generate random values
      const a = Math.floor(Math.random() * 1000000);
      const b = Math.floor(Math.random() * 1000000);
      
      // Check a + b = b + a
      const sum1 = await calculator.add(a, b);
      const sum2 = await calculator.add(b, a);
      
      expect(sum1).to.equal(sum2);
    }
  });
  
  it("multiply has correct relationship with add", async function() {
    for (let i = 0; i < 50; i++) {
      // Generate random value
      const a = Math.floor(Math.random() * 1000);
      
      // Test a * 2 = a + a
      const product = await calculator.multiply(a, 2);
      const sum = await calculator.add(a, a);
      
      expect(product).to.equal(sum);
    }
  });
});
```

### Parameterized Tests

Running tests with multiple parameter sets:

```javascript
// test/unit/advanced/ParameterizedTests.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ValidatorRegistry", function() {
  let validatorRegistry;
  
  before(async function() {
    const ValidatorRegistry = await ethers.getContractFactory("ValidatorRegistry");
    validatorRegistry = await ValidatorRegistry.deploy();
  });
  
  describe("stake requirements", function() {
    // Test parameters
    const testCases = [
      { role: "basic", expectedStake: ethers.utils.parseEther("1000") },
      { role: "advanced", expectedStake: ethers.utils.parseEther("5000") },
      { role: "professional", expectedStake: ethers.utils.parseEther("10000") },
      { role: "enterprise", expectedStake: ethers.utils.parseEther("50000") }
    ];
    
    for (const { role, expectedStake } of testCases) {
      it(`requires correct stake amount for ${role} role`, async function() {
        const requiredStake = await validatorRegistry.getRequiredStake(role);
        expect(requiredStake).to.equal(expectedStake);
      });
    }
  });
  
  describe("score calculation", function() {
    // Test parameters
    const testCases = [
      { stake: "1000", uptime: 100, expectedScore: 100 },
      { stake: "5000", uptime: 100, expectedScore: 150 },
      { stake: "5000", uptime: 95, expectedScore: 142 },
      { stake: "10000", uptime: 100, expectedScore: 200 },
      { stake: "50000", uptime: 100, expectedScore: 300 }
    ];
    
    for (const { stake, uptime, expectedScore } of testCases) {
      it(`calculates correct score for ${stake} stake and ${uptime}% uptime`, async function() {
        const score = await validatorRegistry.calculateScore(
          ethers.utils.parseEther(stake),
          uptime
        );
        
        // Allow slight variation due to integer division
        expect(score).to.be.closeTo(expectedScore, 1);
      });
    }
  });
});
```

### Testing Contract Interactions

Testing contracts that interact with each other:

```javascript
// test/unit/advanced/ContractInteractions.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Protocol interactions", function() {
  let token;
  let staking;
  let rewards;
  let governor;
  let owner;
  let user;
  
  before(async function() {
    [owner, user] = await ethers.getSigners();
    
    // Deploy token
    const Token = await ethers.getContractFactory("Token");
    token = await Token.deploy("Governance Token", "GOV", 1000000);
    
    // Deploy staking contract
    const Staking = await ethers.getContractFactory("Staking");
    staking = await Staking.deploy(token.address);
    
    // Deploy rewards contract
    const Rewards = await ethers.getContractFactory("Rewards");
    rewards = await Rewards.deploy(token.address, staking.address);
    
    // Deploy governor
    const Governor = await ethers.getContractFactory("Governor");
    governor = await Governor.deploy(token.address, staking.address);
    
    // Configure staking contract to work with rewards
    await staking.setRewardsContract(rewards.address);
    
    // Transfer tokens to user
    await token.transfer(user.address, ethers.utils.parseEther("1000"));
  });
  
  it("staking tokens increases voting power", async function() {
    // Initial state
    const initialVotes = await governor.getVotes(user.address);
    expect(initialVotes).to.equal(0);
    
    // User stakes tokens
    await token.connect(user).approve(staking.address, ethers.utils.parseEther("500"));
    await staking.connect(user).stake(ethers.utils.parseEther("500"));
    
    // Check voting power is updated
    const newVotes = await governor.getVotes(user.address);
    expect(newVotes).to.equal(ethers.utils.parseEther("500"));
  });
  
  it("staked tokens earn rewards", async function() {
    // Fund rewards contract
    await token.transfer(rewards.address, ethers.utils.parseEther("10000"));
    
    // Start reward period
    await rewards.startRewardPeriod(ethers.utils.parseEther("1000"), 7 * 24 * 60 * 60); // 1000 tokens over 1 week
    
    // Advance time
    await ethers.provider.send("evm_increaseTime", [24 * 60 * 60]); // 1 day
    await ethers.provider.send("evm_mine");
    
    // Check rewards
    const pendingRewards = await rewards.getPendingRewards(user.address);
    
    // Should have earned ~1/7 of rewards (slight variation due to block timing)
    expect(pendingRewards).to.be.closeTo(
      ethers.utils.parseEther("71.42"), // ~500 / total stake * 1000 / 7
      ethers.utils.parseEther("1")
    );
  });
});
```

## Best Practices

### Naming Conventions

Follow consistent test naming:

- Use descriptive test names that explain what is being tested
- Follow a pattern: "should [expected behavior] when [condition]"
- Be consistent with test file organization

```javascript
// Good test names
it("should revert when caller is not owner", async function() { ... });
it("transfers tokens correctly between accounts", async function() { ... });

// Bad test names
it("test1", async function() { ... });
it("when non-owner calls function", async function() { ... });
```

### Test Isolation

Ensure each test is independent:

- Reset state between tests
- Don't rely on state changes from previous tests
- Use `beforeEach` to set up fresh test conditions

```javascript
describe("Isolated tests", function() {
  // Fresh instance for each test
  beforeEach(async function() {
    // Reset contract state
    token = await Token.deploy("Test Token", "TST", 1000000);
  });
  
  it("first test", async function() {
    await token.transfer(user1.address, 100);
    expect(await token.balanceOf(user1.address)).to.equal(100);
  });
  
  it("second test starts fresh", async function() {
    // No tokens transferred yet in this test
    expect(await token.balanceOf(user1.address)).to.equal(0);
  });
});
```

### Effective Assertions

Write meaningful assertions:

- Test one concept per test
- Use descriptive error messages
- Test both positive and negative cases
- Consider edge cases

```javascript
// Specific assertions with clear error messages
expect(
  await token.balanceOf(recipient.address),
  "Recipient balance should increase by transfer amount"
).to.equal(initialBalance.add(transferAmount));

expect(
  await token.balanceOf(sender.address),
  "Sender balance should decrease by transfer amount"
).to.equal(initialSenderBalance.sub(transferAmount));
```

### Gas Optimization Testing

Measuring and optimizing gas usage:

```javascript
// test/unit/optimizations/GasUsage.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Gas optimizations", function() {
  let optimizedToken;
  let standardToken;
  let owner;
  let recipient;
  
  before(async function() {
    [owner, recipient] = await ethers.getSigners();
    
    // Deploy optimized version
    const OptimizedToken = await ethers.getContractFactory("OptimizedToken");
    optimizedToken = await OptimizedToken.deploy("Optimized", "OPT", 1000000);
    
    // Deploy standard version
    const StandardToken = await ethers.getContractFactory("StandardToken");
    standardToken = await StandardToken.deploy("Standard", "STD", 1000000);
  });
  
  it("optimized transfer uses less gas", async function() {
    // Track gas usage for standard implementation
    const standardTx = await standardToken.transfer(recipient.address, 1000);
    const standardReceipt = await standardTx.wait();
    const standardGas = standardReceipt.gasUsed;
    
    // Track gas usage for optimized implementation
    const optimizedTx = await optimizedToken.transfer(recipient.address, 1000);
    const optimizedReceipt = await optimizedTx.wait();
    const optimizedGas = optimizedReceipt.gasUsed;
    
    console.log(`Standard implementation: ${standardGas.toString()} gas`);
    console.log(`Optimized implementation: ${optimizedGas.toString()} gas`);
    console.log(`Savings: ${standardGas.sub(optimizedGas).toString()} gas`);
    
    // Optimized should use less gas
    expect(optimizedGas).to.be.lt(standardGas);
  });
});
```

## Conclusion

Unit testing forms the critical foundation of blockchain application testing. By thoroughly testing each component in isolation, developers can identify issues early, prevent regressions, and build more robust smart contracts.

The techniques covered in this chapter—from mocking dependencies to testing state transitions—provide a solid basis for creating comprehensive unit tests for ProzChain applications. These tests serve as a safety net for future development, giving developers confidence to refactor and enhance their code without introducing bugs.

In the next chapter, we'll explore integration testing, which builds upon solid unit tests to verify interactions between multiple components.

