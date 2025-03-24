# Testing Best Practices

## Overview

Effective testing is essential for building reliable blockchain applications. This chapter outlines best practices for designing, implementing, and maintaining tests for ProzChain projects. Following these guidelines will help create a comprehensive, maintainable test suite that provides confidence in your code's correctness and security.

## Test Design Principles

### Write Tests First

Embrace test-driven development (TDD) to improve design and catch issues early:

```javascript
// Example of TDD for a token minting function
describe("Token Minting", function() {
  it("should mint tokens and increase total supply", async function() {
    // 1. Write the test first
    const Token = await ethers.getContractFactory("Token");
    const token = await Token.deploy("Test Token", "TST");
    await token.deployed();
    
    const initialSupply = await token.totalSupply();
    const mintAmount = ethers.utils.parseEther("1000");
    const recipient = accounts[1].address;
    
    await token.mint(recipient, mintAmount);
    
    // Assertions drive implementation requirements
    const finalSupply = await token.totalSupply();
    expect(finalSupply.sub(initialSupply)).to.equal(mintAmount);
    expect(await token.balanceOf(recipient)).to.equal(mintAmount);
  });
});
```

### Test One Thing at a Time

Each test should verify a single behavior to make failures easier to diagnose:

```javascript
// Good: One test per behavior
describe("Token Transfers", function() {
  it("should transfer tokens between accounts", async function() {
    // Test only the transfer functionality
  });
  
  it("should fail when sender has insufficient balance", async function() {
    // Test only the insufficient balance case
  });
  
  it("should emit Transfer event with correct parameters", async function() {
    // Test only the event emission
  });
});

// Avoid: Testing multiple behaviors in one test
describe("Token", function() {
  it("should handle transfers, balances, and events correctly", async function() {
    // Testing too many things makes failures harder to diagnose
  });
});
```

### Test Edge Cases

Include tests for boundary conditions and edge cases:

```javascript
describe("Voting System", function() {
  it("should allow voting with minimum required tokens", async function() {
    // Test exact minimum threshold
    const minimumTokens = await votingSystem.getMinimumTokens();
    await token.mint(accounts[1].address, minimumTokens);
    
    await token.connect(accounts[1]).approve(votingSystem.address, minimumTokens);
    await expect(
      votingSystem.connect(accounts[1]).vote(proposalId, true)
    ).to.not.be.reverted;
  });
  
  it("should reject voting with less than minimum tokens", async function() {
    // Test just below minimum threshold
    const minimumTokens = await votingSystem.getMinimumTokens();
    const lessThanMinimum = minimumTokens.sub(1);
    await token.mint(accounts[1].address, lessThanMinimum);
    
    await token.connect(accounts[1]).approve(votingSystem.address, lessThanMinimum);
    await expect(
      votingSystem.connect(accounts[1]).vote(proposalId, true)
    ).to.be.revertedWith("Insufficient voting tokens");
  });
  
  it("should handle maximum uint256 token amount", async function() {
    // Test maximum possible value
    const maxTokens = ethers.constants.MaxUint256;
    await token.mint(accounts[1].address, maxTokens);
    
    await token.connect(accounts[1]).approve(votingSystem.address, maxTokens);
    await expect(
      votingSystem.connect(accounts[1]).vote(proposalId, true)
    ).to.not.be.reverted;
  });
});
```

### Arrange, Act, Assert

Structure tests using the AAA pattern for clarity:

```javascript
it("should update staking rewards when rates change", async function() {
  // Arrange: Set up the test conditions
  await stakingContract.deposit(ethers.utils.parseEther("100"));
  const initialRewardRate = await stakingContract.rewardRate();
  const initialRewards = await stakingContract.rewards(accounts[0].address);
  
  // Advance time to accrue some rewards
  await ethers.provider.send("evm_increaseTime", [86400]); // 1 day
  await ethers.provider.send("evm_mine");
  
  // Act: Perform the action being tested
  const newRewardRate = initialRewardRate.mul(2); // Double the rate
  await stakingContract.setRewardRate(newRewardRate);
  
  // Assert: Verify the outcome
  const updatedRewards = await stakingContract.rewards(accounts[0].address);
  expect(updatedRewards).to.be.gt(initialRewards);
  expect(await stakingContract.rewardRate()).to.equal(newRewardRate);
});
```

## Maintainable Test Code

### Use Descriptive Test Names

Write test descriptions that clearly communicate intent:

```javascript
// Good: Clear test descriptions
it("should revert when non-owner attempts to upgrade contract", async function() {
  // Test implementation
});

// Avoid: Vague descriptions
it("should work correctly", async function() {
  // What does "work correctly" mean?
});
```

### DRY Test Setup

Use hooks and fixtures to reduce repetition:

```javascript
describe("Staking System", function() {
  let token;
  let stakingContract;
  let owner;
  let user1;
  let user2;
  
  // Common setup for all tests
  beforeEach(async function() {
    [owner, user1, user2] = await ethers.getSigners();
    
    const Token = await ethers.getContractFactory("RewardToken");
    token = await Token.deploy("Reward", "RWD");
    await token.deployed();
    
    const Staking = await ethers.getContractFactory("StakingContract");
    stakingContract = await Staking.deploy(token.address);
    await stakingContract.deployed();
    
    // Mint tokens to users
    const mintAmount = ethers.utils.parseEther("1000");
    await token.mint(user1.address, mintAmount);
    await token.mint(user2.address, mintAmount);
    
    // Approve staking contract
    await token.connect(user1).approve(stakingContract.address, mintAmount);
    await token.connect(user2).approve(stakingContract.address, mintAmount);
  });
  
  it("should allow staking tokens", async function() {
    // Test-specific code
  });
  
  it("should calculate rewards correctly", async function() {
    // Test-specific code
  });
  
  // More tests...
});
```

### Use Helper Functions

Extract common test operations into helper functions:

```javascript
// Helper functions for common test operations
function advanceBlockTimestamp(seconds) {
  return ethers.provider.send("evm_increaseTime", [seconds]);
}

async function mineBlock() {
  return ethers.provider.send("evm_mine");
}

async function advanceTimeAndBlock(seconds) {
  await advanceBlockTimestamp(seconds);
  await mineBlock();
}

async function deployTokenAndStaking() {
  const Token = await ethers.getContractFactory("RewardToken");
  const token = await Token.deploy("Reward", "RWD");
  
  const Staking = await ethers.getContractFactory("StakingContract");
  const staking = await Staking.deploy(token.address);
  
  return { token, staking };
}

// Use helpers in tests
it("should accrue rewards over time", async function() {
  const { token, staking } = await deployTokenAndStaking();
  
  await token.mint(accounts[0].address, ethers.utils.parseEther("100"));
  await token.approve(staking.address, ethers.utils.parseEther("100"));
  await staking.stake(ethers.utils.parseEther("100"));
  
  // Advance time by 30 days
  await advanceTimeAndBlock(30 * 24 * 60 * 60);
  
  const rewards = await staking.pendingRewards(accounts[0].address);
  expect(rewards).to.be.gt(0);
});
```

### Isolate Tests

Ensure tests don't depend on each other or external state:

```javascript
// Bad: Tests depend on each other
let sharedState;

it("first test modifies state", async function() {
  sharedState = await someFunction();
  // Assertions...
});

it("second test depends on modified state", async function() {
  // This will fail if run independently
  await doSomethingWith(sharedState);
  // Assertions...
});

// Good: Each test is self-contained
it("first test", async function() {
  const localState = await someFunction();
  // Assertions...
});

it("second test is independent", async function() {
  const localState = await someFunction();
  await doSomethingWith(localState);
  // Assertions...
});
```

### Mock External Dependencies

Isolate tests from external systems:

```javascript
describe("Oracle Consumer", function() {
  let oracleConsumer;
  let mockOracle;
  
  beforeEach(async function() {
    // Deploy mock instead of using real oracle
    const MockOracle = await ethers.getContractFactory("MockPriceOracle");
    mockOracle = await MockOracle.deploy();
    
    const OracleConsumer = await ethers.getContractFactory("OracleConsumer");
    oracleConsumer = await OracleConsumer.deploy(mockOracle.address);
  });
  
  it("should fetch and process price data", async function() {
    // Configure mock to return specific value
    await mockOracle.setPrice("ETH", ethers.utils.parseUnits("1500", 8));
    
    // Test consumer's interaction with the oracle
    const ethPrice = await oracleConsumer.getTokenPrice("ETH");
    expect(ethPrice).to.equal(ethers.utils.parseUnits("1500", 8));
  });
  
  it("should handle oracle failures gracefully", async function() {
    // Configure mock to simulate failure
    await mockOracle.setSimulateFailure(true);
    
    // Verify our consumer handles errors properly
    await expect(
      oracleConsumer.getTokenPrice("ETH")
    ).to.be.revertedWith("Oracle error");
  });
});
```

## Test Documentation

### Document Test Purpose

Add comments explaining complex test cases:

```javascript
/**
 * Tests the flash loan functionality under various conditions.
 * This test verifies that:
 * 1. Users can borrow funds if they return them in the same transaction
 * 2. The contract correctly charges and collects fees
 * 3. The contract prevents flash loan reentrancy attacks
 */
it("should execute flash loans correctly and securely", async function() {
  // Implementation...
});
```

### Document Test Coverage Gaps

Make intentional gaps in test coverage explicit:

```javascript
describe("Complex Staking Functionality", function() {
  // Standard tests...
  
  /**
   * @notice The following scenarios are not currently tested:
   * - Extremely large stakes (> 10^30 tokens) due to gas limitations
   * - Edge cases with reward calculation over multi-year periods
   * - Migration from v1 to v2 staking contract (tested in migration test suite)
   */
  
  it("handles normal staking operations", async function() {
    // Implementation...
  });
});
```

### Document Test Requirements

Include information on required environment setup:

```javascript
/**
 * These tests require:
 * - Local hardhat node with mainnet fork
 * - FORKED_BLOCK_NUMBER environment variable set to the block number used for forking
 * - ALCHEMY_API_KEY environment variable for the Alchemy API
 * 
 * Run with:
 * FORKED_BLOCK_NUMBER=15000000 ALCHEMY_API_KEY=your-key npx hardhat test
 */
describe("Mainnet Integration Tests", function() {
  before(async function() {
    if (!process.env.FORKED_BLOCK_NUMBER || !process.env.ALCHEMY_API_KEY) {
      this.skip();
    }
  });
  
  it("should work with actual mainnet contracts", async function() {
    // Implementation...
  });
});
```

## Common Testing Pitfalls

### Non-Deterministic Tests

Avoid non-deterministic elements in tests:

```javascript
// Bad: Non-deterministic test
it("should randomly distribute rewards", async function() {
  const randomReward = await rewardSystem.claimRandomReward();
  // Could sometimes pass, sometimes fail!
  expect(randomReward).to.be.gt(0);
});

// Good: Control randomness in tests
it("should distribute rewards based on pseudo-random values", async function() {
  // Mock the randomness source to return a known value
  await randomnessProvider.setNextRandomValue(42);
  
  const reward = await rewardSystem.claimRandomReward();
  
  // We know exactly what reward to expect given randomness of 42
  const expectedReward = calculateRewardForRandom(42);
  expect(reward).to.equal(expectedReward);
});
```

### Gas Estimation Issues

Account for gas usage variations:

```javascript
// Bad: Hard-coded gas limit that might change
it("should execute within gas limits", async function() {
  const tx = await contract.complexOperation();
  const receipt = await tx.wait();
  expect(receipt.gasUsed).to.be.lt(1000000);
});

// Good: Relative gas measurement
it("should not increase gas usage significantly", async function() {
  // Establish baseline
  const baselineTx = await contract.simpleOperation();
  const baselineReceipt = await baselineTx.wait();
  
  // Measure actual operation
  const tx = await contract.complexOperation();
  const receipt = await tx.wait();
  
  // Allow some reasonable multiple of the baseline
  expect(receipt.gasUsed).to.be.lt(baselineReceipt.gasUsed * 5);
  
  // Log gas usage for tracking over time
  console.log(`Gas used: ${receipt.gasUsed.toNumber()}`);
});
```

### Incorrect Cleanup

Ensure proper test cleanup:

```javascript
describe("State management", function() {
  // Using nested before/after hooks for proper cleanup
  before(async function() {
    // Global setup
    this.snapshot = await ethers.provider.send("evm_snapshot", []);
  });
  
  after(async function() {
    // Global cleanup
    await ethers.provider.send("evm_revert", [this.snapshot]);
  });
  
  // Each test can have its own setup/cleanup
  it("modifies state", async function() {
    const localSnapshot = await ethers.provider.send("evm_snapshot", []);
    
    // Test that modifies state...
    
    // Cleanup after this specific test
    await ethers.provider.send("evm_revert", [localSnapshot]);
  });
});
```

### Insufficient Test Coverage

Address critical code paths:

```javascript
describe("Critical Functionality", function() {
  // Test basic functionality
  it("should allow normal transfers", async function() {
    // Basic test...
  });
  
  // Test edge cases and error conditions
  it("should handle zero transfers", async function() {
    // Edge case test...
  });
  
  it("should reject transfers exceeding balance", async function() {
    // Error condition test...
  });
  
  // Test security aspects
  it("should prevent unauthorized withdrawals", async function() {
    // Security test...
  });
  
  // Test business logic constraints
  it("should enforce transfer limits", async function() {
    // Business rule test...
  });
});

// Avoid: Testing only the happy path
describe("Insufficient Coverage", function() {
  it("should process transactions", async function() {
    // Only tests the basic case...
  });
});
```

## Testing in Different Environments

### Local Development

```javascript
describe("Local Development Tests", function() {
  it("should run quickly for rapid feedback", async function() {
    // Fast tests for local development
  });
});
```

### CI Environment

```javascript
describe("CI Environment Tests", function() {
  before(function() {
    if (!process.env.CI) {
      console.log("These tests are designed for CI environments");
    }
  });
  
  it("should run comprehensive test suite", async function() {
    // More thorough tests for CI
  });
});
```

### Production-Like Environment

```javascript
describe("Production-Like Tests", function() {
  before(function() {
    if (!process.env.PROD_TEST_ENV) {
      this.skip();
    }
  });
  
  it("should test with realistic data volumes", async function() {
    // Tests with production-like data sizes
  });
});
```

## Continuous Improvement

Regularly review and improve tests:

```javascript
// Test retrospective notes (as code comments)
/**
 * Test Improvement Notes:
 * 
 * 1. We found that our gas usage tests were brittle due to compiler optimizations.
 *    Solution: Use relative comparisons instead of absolute gas limits.
 * 
 * 2. Integration tests were flaky when run in parallel.
 *    Solution: Added proper resource cleanup and isolation.
 * 
 * 3. Mock implementations diverged from real components over time.
 *    Solution: Added validation tests for mocks against real implementations.
 */
```

## Conclusion

Effective testing practices are essential for developing reliable, maintainable blockchain applications. By following the best practices outlined in this chapter—writing clear, focused tests, ensuring proper isolation, and maintaining comprehensive coverage—you can build a test suite that provides confidence in your code's correctness and helps catch issues early in the development process.

Remember that testing is an investment in code quality and development productivity. The time spent on creating good tests pays dividends in reduced debugging time, fewer production issues, and easier refactoring and enhancement of your codebase.
