# Security Testing

## Overview

Security testing is critical for blockchain applications, where vulnerabilities can lead to significant financial losses and reputation damage. This chapter explores methodologies, tools, and best practices for conducting thorough security testing on ProzChain applications. Security testing for blockchain applications requires a specialized approach that addresses smart contract vulnerabilities, cryptographic weaknesses, and economic attack vectors.

By implementing a comprehensive security testing strategy, developers can identify and mitigate potential security risks before deployment, reducing the likelihood of exploits and ensuring the integrity of blockchain applications.

## Vulnerability Scanning

### Automated Security Tools

Tools for finding common smart contract vulnerabilities:

1. **Static Analysis Tools**:
   - Slither: Comprehensive static analysis framework
   - Mythril: Symbolic execution and formal verification
   - Securify: Pattern-based vulnerability scanner
   - Solhint: Linting for security best practices

2. **Example: Setting up Slither**:

```bash
# Install Slither
pip install slither-analyzer

# Run basic scan on a contract
slither contracts/Token.sol

# Generate detailed JSON report
slither contracts/Token.sol --json report.json

# Check for specific vulnerability detectors
slither contracts/Token.sol --detect reentrancy,arbitrary-send
```

3. **Example: Integrating with CI Pipeline**:

```yaml
# Security scanning job in CI
security-scan:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - name: Setup Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.9'
    - name: Install Slither
      run: pip install slither-analyzer
    - name: Run Slither
      run: slither . --json slither-report.json
    - name: Check for high severity findings
      run: |
        HIGH_FINDINGS=$(jq '.results.detectors[] | select(.impact == "High") | .elements' slither-report.json | jq length)
        if [ $HIGH_FINDINGS -gt 0 ]; then
          echo "Found $HIGH_FINDINGS high severity issues!"
          exit 1
        fi
```

### Common Vulnerability Patterns

Understanding and testing for common smart contract weaknesses:

1. **Reentrancy Vulnerabilities**:
   - Detection techniques
   - Testing for reentrancy conditions
   - Mitigation strategies
   - Check-Effects-Interaction pattern validation

2. **Example: Reentrancy Test**:

```javascript
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Reentrancy Protection", function() {
  let vulnerableContract;
  let attackerContract;
  let owner, attacker;
  
  beforeEach(async function() {
    [owner, attacker] = await ethers.getSigners();
    
    // Deploy vulnerable contract
    const VulnerableContract = await ethers.getContractFactory("VulnerableBank");
    vulnerableContract = await VulnerableContract.deploy();
    await vulnerableContract.deposit({ value: ethers.utils.parseEther("10") });
    
    // Deploy attacker contract
    const AttackerContract = await ethers.getContractFactory("ReentrancyAttacker");
    attackerContract = await AttackerContract.deploy(vulnerableContract.address);
  });
  
  it("should be vulnerable to reentrancy", async function() {
    // Fund attacker contract
    await attacker.sendTransaction({
      to: attackerContract.address,
      value: ethers.utils.parseEther("1")
    });
    
    // Check initial balances
    const initialVulnerableBalance = await ethers.provider.getBalance(vulnerableContract.address);
    const initialAttackerBalance = await ethers.provider.getBalance(attackerContract.address);
    
    // Execute attack
    await attackerContract.connect(attacker).attack();
    
    // Check final balances
    const finalVulnerableBalance = await ethers.provider.getBalance(vulnerableContract.address);
    const finalAttackerBalance = await ethers.provider.getBalance(attackerContract.address);
    
    // Verify that attacker was able to drain more than deposited
    expect(finalAttackerBalance.sub(initialAttackerBalance))
      .to.be.gt(ethers.utils.parseEther("1"));
    
    // Verify vulnerable contract lost funds
    expect(finalVulnerableBalance).to.be.lt(initialVulnerableBalance);
  });
  
  it("should resist reentrancy after fix", async function() {
    // Deploy fixed contract
    const SecureContract = await ethers.getContractFactory("SecureBank");
    const secureContract = await SecureContract.deploy();
    await secureContract.deposit({ value: ethers.utils.parseEther("10") });
    
    // Deploy attacker targeting secure contract
    const SecureAttacker = await ethers.getContractFactory("ReentrancyAttacker");
    const secureAttacker = await SecureAttacker.deploy(secureContract.address);
    
    // Fund attacker
    await attacker.sendTransaction({
      to: secureAttacker.address,
      value: ethers.utils.parseEther("1")
    });
    
    // Attempt attack
    await expect(
      secureAttacker.connect(attacker).attack()
    ).to.be.reverted;
    
    // Verify balances unchanged (except for gas costs)
    expect(await ethers.provider.getBalance(secureContract.address))
      .to.equal(ethers.utils.parseEther("10"));
  });
});
```

3. **Access Control Vulnerabilities**:
   - Unauthorized access tests
   - Missing/incorrect permission checks
   - Privilege escalation vectors
   - Role-based access enforcement

4. **Integer Overflow and Underflow**:
   - Boundary testing for mathematical operations
   - SafeMath usage verification
   - Edge case simulations
   - Compiler version validation

5. **Example: Integer Overflow Test**:

```javascript
describe("Integer Vulnerability Tests", function() {
  let mathContract;
  
  beforeEach(async function() {
    const MathContract = await ethers.getContractFactory("MathOperations");
    mathContract = await MathContract.deploy();
  });
  
  it("should handle uint256 overflow correctly", async function() {
    const maxUint = ethers.constants.MaxUint256;
    
    // Test unsecured add function (vulnerable to overflow)
    await expect(
      mathContract.unsecuredAdd(maxUint, 1)
    ).to.not.be.reverted; // Will wrap around to 0 silently
    
    // Verify incorrect result due to overflow
    expect(await mathContract.getLastResult()).to.equal(0);
    
    // Test secured add function (prevents overflow)
    await expect(
      mathContract.securedAdd(maxUint, 1)
    ).to.be.revertedWith("arithmetic overflow");
  });
  
  it("should handle uint256 underflow correctly", async function() {
    // Test unsecured subtract function (vulnerable to underflow)
    await expect(
      mathContract.unsecuredSubtract(0, 1)
    ).to.not.be.reverted; // Will wrap around to MaxUint256
    
    // Verify incorrect result due to underflow
    expect(await mathContract.getLastResult()).to.equal(ethers.constants.MaxUint256);
    
    // Test secured subtract function (prevents underflow)
    await expect(
      mathContract.securedSubtract(0, 1)
    ).to.be.revertedWith("arithmetic underflow");
  });
});
```

## Penetration Testing

### Manual Security Reviews

Conducting thorough security audits:

1. **Code Review Process**:
   - Line-by-line code examination
   - Logic flow analysis
   - State transition security
   - Access control verification

2. **Security Review Checklist**:

```javascript
// Security review checklist implementation
const securityReview = {
  accessControl: [
    "✓ Only authorized roles can access administrative functions",
    "✓ Role management follows principle of least privilege",
    "✓ Two-step ownership transfer pattern implemented",
    "✓ Emergency access controls properly protected",
    "✓ No privileged functions exposed without access control"
  ],
  
  dataValidation: [
    "✓ All external inputs validated",
    "✓ Array inputs have reasonable length limits",
    "✓ Numeric inputs checked for valid ranges",
    "✓ Address inputs validated (non-zero check)",
    "✓ Defense against signature replay attacks"
  ],
  
  fundsSecurity: [
    "✓ No direct contract balance manipulation",
    "✓ Pull over push payment pattern used",
    "✓ No unchecked send/transfer operations",
    "✓ Reentrancy guards on all fund-transferring functions",
    "✓ Balance accounting consistent with token transfers"
  ],
  
  codeQuality: [
    "✓ No use of deprecated functions (tx.origin, suicide, etc.)",
    "✓ Gas-efficient loops with bounds checking",
    "✓ Safe mathematical operations",
    "✓ Error handling with informative revert messages",
    "✓ Events emitted for all important state changes"
  ],
  
  externalInteractions: [
    "✓ Safe handling of external contract calls",
    "✓ Low-level calls followed by success checks",
    "✓ Handling of non-conforming contracts",
    "✓ Oracle manipulation resistance",
    "✓ Reasonable gas limits for external calls"
  ]
};
```

3. **Expert Audit Coordination**:
   - Preparing for external security audits
   - Scope definition
   - Finding triage and remediation
   - Post-audit verification testing

### Attack Vector Simulation

Testing against known attack vectors:

1. **Front-Running Protection**:
   - Commit-reveal schemes testing
   - MEV resistance validation
   - Transaction ordering dependency tests
   - Timestamping mechanism verification

2. **Example: Front-Running Test**:

```javascript
describe("Front-Running Protection", function() {
  let auction;
  let owner, bidder1, bidder2, frontRunner;
  
  beforeEach(async function() {
    [owner, bidder1, bidder2, frontRunner] = await ethers.getSigners();
    
    // Deploy vulnerable auction
    const VulnerableAuction = await ethers.getContractFactory("VulnerableAuction");
    vulnAuction = await VulnerableAuction.deploy();
    
    // Deploy protected auction with commit-reveal scheme
    const SecureAuction = await ethers.getContractFactory("CommitRevealAuction");
    secureAuction = await SecureAuction.deploy();
  });
  
  it("demonstrates front-running vulnerability", async function() {
    // Bidder1 prepares bid but doesn't submit yet
    const bidder1Bid = ethers.utils.parseEther("1.0");
    
    // Front-runner monitors mempool and sees the pending transaction
    // Simulated here by creating the transaction but not sending it
    const bidder1Tx = await vulnAuction.populateTransaction.placeBid({
      value: bidder1Bid
    });
    
    // Front-runner submits with slightly higher price and more gas
    const frontRunBid = bidder1Bid.add(ethers.utils.parseEther("0.1"));
    await frontRunner.sendTransaction({
      to: vulnAuction.address,
      value: frontRunBid,
      gasPrice: (await ethers.provider.getGasPrice()).mul(2),
      data: bidder1Tx.data
    });
    
    // Bidder1's transaction goes through after front-runner
    await bidder1.sendTransaction({
      to: vulnAuction.address,
      value: bidder1Bid,
      data: bidder1Tx.data
    });
    
    // Verify front-runner is highest bidder
    expect(await vulnAuction.highestBidder()).to.equal(frontRunner.address);
  });
  
  it("resists front-running with commit-reveal scheme", async function() {
    // Bidder creates bid commitment (hash of bid value + secret)
    const bidValue = ethers.utils.parseEther("1.0");
    const secret = ethers.utils.formatBytes32String("mysecret");
    const bidCommitment = ethers.utils.keccak256(
      ethers.utils.defaultAbiCoder.encode(
        ["address", "uint256", "bytes32"],
        [bidder1.address, bidValue, secret]
      )
    );
    
    // Bidder submits commitment
    await secureAuction.connect(bidder1).commitBid(bidCommitment);
    
    // Front-runner can't determine the actual bid value
    const frontRunGuess = ethers.utils.parseEther("1.1");
    const frontRunSecret = ethers.utils.formatBytes32String("guessed");
    const frontRunCommitment = ethers.utils.keccak256(
      ethers.utils.defaultAbiCoder.encode(
        ["address", "uint256", "bytes32"],
        [frontRunner.address, frontRunGuess, frontRunSecret]
      )
    );
    await secureAuction.connect(frontRunner).commitBid(frontRunCommitment);
    
    // Move to reveal phase
    await secureAuction.connect(owner).startRevealPhase();
    
    // Bidder reveals actual bid
    await secureAuction.connect(bidder1).revealBid(bidValue, secret, {
      value: bidValue
    });
    
    // Front-runner reveals guess
    await secureAuction.connect(frontRunner).revealBid(frontRunGuess, frontRunSecret, {
      value: frontRunGuess
    });
    
    // Verify correct highest bidder regardless of transaction order
    expect(await secureAuction.highestBidder()).to.equal(frontRunner.address);
    expect(await secureAuction.highestBid()).to.equal(frontRunGuess);
  });
});
```

3. **Oracle Manipulation**:
   - Price oracle attack simulations
   - Data feed manipulation tests
   - Flash loan attack simulations
   - Time-weighted average price validations

4. **Example: Oracle Security Test**:

```javascript
describe("Oracle Security", function() {
  let lending;
  let mockToken;
  let mockOracle;
  let owner, attacker, user;
  
  beforeEach(async function() {
    [owner, attacker, user] = await ethers.getSigners();
    
    // Deploy mock token
    const MockToken = await ethers.getContractFactory("MockToken");
    mockToken = await MockToken.deploy("Mock Token", "MTK");
    
    // Deploy vulnerable oracle
    const VulnerableOracle = await ethers.getContractFactory("SingleSourceOracle");
    vulnOracle = await VulnerableOracle.deploy(mockToken.address);
    
    // Deploy secure oracle
    const SecureOracle = await ethers.getContractFactory("MedianOracle");
    secureOracle = await SecureOracle.deploy(mockToken.address);
    
    // Add price sources to secure oracle
    await secureOracle.addSource(owner.address, "Source1");
    await secureOracle.addSource(user.address, "Source2");
    await secureOracle.addSource(ethers.constants.AddressZero, "Source3"); // System source
    
    // Submit initial prices
    await vulnOracle.submitPrice(ethers.utils.parseEther("100"));
    await secureOracle.submitPrice(ethers.utils.parseEther("100"));
    await secureOracle.connect(user).submitPrice(ethers.utils.parseEther("101"));
    
    // Deploy lending protocol using each oracle
    const VulnerableLending = await ethers.getContractFactory("LendingProtocol");
    vulnLending = await VulnerableLending.deploy(mockToken.address, vulnOracle.address);
    
    const SecureLending = await ethers.getContractFactory("LendingProtocol");
    secureLending = await SecureLending.deploy(mockToken.address, secureOracle.address);
    
    // Mint tokens to user and attacker
    await mockToken.mint(user.address, ethers.utils.parseEther("1000"));
    await mockToken.mint(attacker.address, ethers.utils.parseEther("10000"));
    
    // Fund lending protocols
    await mockToken.mint(vulnLending.address, ethers.utils.parseEther("10000"));
    await mockToken.mint(secureLending.address, ethers.utils.parseEther("10000"));
    
    // Approve spending
    await mockToken.connect(user).approve(vulnLending.address, ethers.utils.parseEther("1000"));
    await mockToken.connect(user).approve(secureLending.address, ethers.utils.parseEther("1000"));
    await mockToken.connect(attacker).approve(vulnLending.address, ethers.utils.parseEther("10000"));
  });
  
  it("demonstrates oracle price manipulation attack", async function() {
    // User deposits collateral in vulnerable lending
    await vulnLending.connect(user).depositCollateral(ethers.utils.parseEther("100"));
    
    // User borrows against collateral (50% LTV)
    await vulnLending.connect(user).borrow(ethers.utils.parseEther("50"));
    
    // Attacker manipulates oracle price downward
    await vulnOracle.connect(attacker).submitPrice(ethers.utils.parseEther("10"));
    
    // Check if user position is now eligible for liquidation
    expect(await vulnLending.canLiquidate(user.address)).to.be.true;
    
    // Attacker liquidates position at manipulated price
    await vulnLending.connect(attacker).liquidate(user.address);
    
    // Verify attacker got collateral at discount
    expect(await mockToken.balanceOf(attacker.address))
      .to.be.gt(ethers.utils.parseEther("10050")); // Initial + borrowed + profit
  });
  
  it("resists oracle manipulation with median pricing", async function() {
    // User deposits collateral in secure lending
    await secureLending.connect(user).depositCollateral(ethers.utils.parseEther("100"));
    
    // User borrows against collateral (50% LTV)
    await secureLending.connect(user).borrow(ethers.utils.parseEther("50"));
    
    // Attacker tries to manipulate one price source downward
    await secureOracle.connect(attacker).submitPrice(ethers.utils.parseEther("10"));
    
    // Median price should remain stable due to other sources
    expect(await secureOracle.getPrice()).to.equal(ethers.utils.parseEther("100"));
    
    // User position should not be liquidatable
    expect(await secureLending.canLiquidate(user.address)).to.be.false;
  });
});
```

### Security Testing with Foundry

Using Foundry framework for security testing:

1. **Fuzzing for Security Vulnerabilities**:
   - Fuzz testing sensitive functions
   - Boundary condition detection
   - Unexpected input handling
   - State transition exploration

2. **Example: Foundry Security Test**:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/Token.sol";

contract SecurityTest is Test {
    Token token;
    address owner = address(1);
    address user = address(2);
    address attacker = address(3);
    
    function setUp() public {
        vm.startPrank(owner);
        token = new Token("Secure Token", "SEC");
        token.mint(owner, 1000000 ether);
        vm.stopPrank();
    }
    
    // Fuzz test to check approval race condition vulnerability
    function testFuzz_ApprovalFrontRunningProtection(uint256 initialAllowance, uint256 newAllowance) public {
        // Bound values to avoid overflows
        initialAllowance = bound(initialAllowance, 1, 1000000 ether);
        newAllowance = bound(newAllowance, 1, 1000000 ether);
        
        // Owner approves initial allowance to attacker
        vm.prank(owner);
        token.approve(attacker, initialAllowance);
        assertEq(token.allowance(owner, attacker), initialAllowance);
        
        // Simulate front-running attack
        // Attacker quickly uses the entire allowance before owner can change it
        vm.prank(attacker);
        token.transferFrom(owner, attacker, initialAllowance);
        
        // Owner tries to change allowance to new value
        vm.prank(owner);
        vm.expectRevert("ERC20: insufficient allowance");
        token.transferFrom(owner, user, newAllowance);
        
        // Verify attacker can't spend more than approved
        vm.prank(attacker);
        vm.expectRevert("ERC20: insufficient allowance");
        token.transferFrom(owner, attacker, 1);
    }
    
    // Test for zero address transfers
    function testFuzz_NoTransferToZeroAddress(uint256 amount) public {
        amount = bound(amount, 1, 1000000 ether);
        
        // Attempt transfer to zero address
        vm.prank(owner);
        vm.expectRevert("ERC20: transfer to zero address");
        token.transfer(address(0), amount);
    }
    
    // Invariant test: total supply should equal sum of all balances
    function invariant_BalanceSumEqualsTotalSupply() public {
        uint256 totalBalances = token.balanceOf(owner) + 
                               token.balanceOf(user) +
                               token.balanceOf(attacker) +
                               token.balanceOf(address(this));
        
        assertEq(token.totalSupply(), totalBalances);
    }
}
```

## Formal Verification

### Code Verification Techniques

Using formal methods to prove contract correctness:

1. **Types of Formal Verification**:
   - Model checking
   - Static verification
   - Theorem proving
   - Symbolic execution

2. **Certora Prover Integration**:

```javascript
// rules.spec
methods {
    // Define contract methods to analyze
    balanceOf(address) returns (uint256) envfree
    totalSupply() returns (uint256) envfree
    transfer(address, uint256) returns (bool)
    transferFrom(address, address, uint256) returns (bool)
    approve(address, uint256) returns (bool)
    allowance(address, address) returns (uint256) envfree
}

// Declare ghost variables to track balances
ghost mathint totalMinted {
    init_state axiom totalMinted == 0;
}

// Rules that define correct behavior
rule balancesSum {
    // Sum of all balances equals totalSupply
    assert sum<address>((a) => balanceOf(a)) == totalSupply();
}

rule transferPreservesTotalSupply {
    env e;
    address to;
    uint256 amount;
    
    uint256 totalSupplyBefore = totalSupply();
    
    transfer(e, to, amount);
    
    uint256 totalSupplyAfter = totalSupply();
    
    assert totalSupplyBefore == totalSupplyAfter;
}

rule transferFromPreservesBalance {
    env e;
    address from;
    address to;
    uint256 amount;
    
    uint256 fromBalanceBefore = balanceOf(from);
    uint256 toBalanceBefore = balanceOf(to);
    
    transferFrom(e, from, to, amount);
    
    uint256 fromBalanceAfter = balanceOf(from);
    uint256 toBalanceAfter = balanceOf(to);
    
    assert fromBalanceAfter == fromBalanceBefore - amount;
    assert toBalanceAfter == toBalanceBefore + amount;
}

rule noTransferToZeroAddress {
    env e;
    uint256 amount;
    
    require amount > 0;
    
    // This should revert
    bool success = transfer@withrevert(e, 0, amount);
    
    assert !success;
}
```

3. **SMT Solvers for Contract Verification**:
   - Z3 integration
   - SMTChecker configuration
   - Contract invariant verification
   - Safety property validation

### Security Properties

Formally specifying and verifying security constraints:

1. **Contract Invariants**:
   - Balance consistency
   - Access control constraints
   - State transition rules
   - Numerical bounds

2. **Verifying Authorization Logic**:

```solidity
// Property-based testing for authorization
import "forge-std/Test.sol";
import "../src/AccessControl.sol";

contract AuthorizationTest is Test {
    AccessControl access;
    address admin = address(1);
    address user = address(2);
    
    bytes32 constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 constant USER_ROLE = keccak256("USER_ROLE");
    
    function setUp() public {
        access = new AccessControl();
        access.initialize(admin);
    }
    
    // Property: Only DEFAULT_ADMIN_ROLE can grant roles
    function testProperty_OnlyAdminCanGrantRoles(address randomAddress) public {
        vm.assume(randomAddress != address(0));
        vm.assume(randomAddress != admin);
        vm.assume(!access.hasRole(access.DEFAULT_ADMIN_ROLE(), randomAddress));
        
        // Random address attempts to grant role, should fail
        vm.startPrank(randomAddress);
        vm.expectRevert("AccessControl: sender must be admin");
        access.grantRole(USER_ROLE, user);
        vm.stopPrank();
        
        // Admin should succeed
        vm.prank(admin);
        access.grantRole(USER_ROLE, user);
        assertTrue(access.hasRole(USER_ROLE, user));
    }
    
    // Property: Role checks are consistent
    function testProperty_RoleChecksConsistent(address account, bytes32 role) public {
        bool hasRoleDirect = access.hasRole(role, account);
        
        // Check if onlyRole modifier would allow the action
        vm.startPrank(account);
        bool actionSucceeded = address(access).call(
            abi.encodeWithSelector(access.protectedAction.selector, role)
        );
        vm.stopPrank();
        
        assertEq(hasRoleDirect, actionSucceeded);
    }
    
    // Property: No dangling admin rights after renouncement
    function testProperty_RenounceRemovesRole(address account, bytes32 role) public {
        // First grant the role
        vm.prank(admin);
        access.grantRole(role, account);
        assertTrue(access.hasRole(role, account));
        
        // Account renounces role
        vm.prank(account);
        access.renounceRole(role, account);
        
        // Verify role was removed
        assertFalse(access.hasRole(role, account));
    }
}
```

## Economic Security Testing

### Game Theory Analysis

Testing incentive structures and attack economics:

1. **Economic Attack Vectors**:
   - Profit-motivated attacks
   - Collusion scenarios
   - Bribery attacks
   - Game theory equilibrium testing

2. **Example: Game Theory Test**:

```javascript
describe("Staking Protocol Economic Security", function() {
  let staking;
  let token;
  let accounts;
  
  beforeEach(async function() {
    accounts = await ethers.getSigners();
    
    // Deploy token
    const Token = await ethers.getContractFactory("StakingToken");
    token = await Token.deploy("Stake", "STK");
    
    // Mint tokens to test accounts
    for (let i = 0; i < 10; i++) {
      await token.mint(accounts[i].address, ethers.utils.parseEther("1000"));
    }
    
    // Deploy staking protocol
    const Staking = await ethers.getContractFactory("StakingProtocol");
    staking = await Staking.deploy(token.address);
    
    // Each account approves staking contract
    for (let i = 0; i < 10; i++) {
      await token.connect(accounts[i]).approve(
        staking.address,
        ethers.utils.parseEther("1000")
      );
    }
  });
  
  it("verifies no early withdrawal advantage", async function() {
    // Scenario: Multiple users stake, one tries to game rewards
    
    // 5 users stake 100 tokens each
    for (let i = 0; i < 5; i++) {
      await staking.connect(accounts[i]).stake(ethers.utils.parseEther("100"));
    }
    
    // Distribute initial rewards
    await staking.distributeRewards(ethers.utils.parseEther("50"));
    
    // Advance time 10 days
    await ethers.provider.send("evm_increaseTime", [10 * 86400]);
    await ethers.provider.send("evm_mine");
    
    // User 0 attempts strategic withdrawal and re-staking
    await staking.connect(accounts[0]).withdraw(ethers.utils.parseEther("100"));
    await staking.connect(accounts[0]).stake(ethers.utils.parseEther("100"));
    
    // Distribute more rewards
    await staking.distributeRewards(ethers.utils.parseEther("50"));
    
    // Advance time 10 more days
    await ethers.provider.send("evm_increaseTime", [10 * 86400]);
    await ethers.provider.send("evm_mine");
    
    // Calculate rewards for strategic user vs. continuous staker
    const strategicUserRewards = await staking.earned(accounts[0].address);
    const continuousStakerRewards = await staking.earned(accounts[1].address);
    
    // Strategic user shouldn't get more rewards than continuous stakers
    expect(strategicUserRewards).to.be.at.most(continuousStakerRewards);
    
    // In fact, they should get fewer rewards due to the time not staked
    expect(strategicUserRewards).to.be.lt(continuousStakerRewards);
  });
  
  it("verifies whale resistance in reward distribution", async function() {
    // Scenario: Test if large stakers can disproportionately affect smaller ones
    
    // Small stakers stake 10 tokens each
    for (let i = 1; i < 5; i++) {
      await staking.connect(accounts[i]).stake(ethers.utils.parseEther("10"));
    }
    
    // Distribute initial rewards
    await staking.distributeRewards(ethers.utils.parseEther("40"));
    
    // Record initial rewards accrual rate per token
    const initialRate = await staking.rewardPerTokenStored();
    
    // Whale stakes a large amount
    await staking.connect(accounts[0]).stake(ethers.utils.parseEther("960"));
    
    // Distribute same amount of rewards
    await staking.distributeRewards(ethers.utils.parseEther("40"));
    
    // Advance time 10 days
    await ethers.provider.send("evm_increaseTime", [10 * 86400]);
    await ethers.provider.send("evm_mine");
    
    // Calculate rewards for a small staker before and after whale joined
    const smallStakerRewards = await staking.earned(accounts[1].address);
    
    // Rewards should be proportionate to stake
    // Each small user has 10/1000 = 1% of the pool after whale joins
    // They should get approximately 1% of the rewards
    const expectedRewards = ethers.utils.parseEther("0.8");  // 40 * 0.01 * 2 distributions
    
    // Allow some margin for small calculation differences
    expect(smallStakerRewards).to.be.closeTo(
      expectedRewards,
      ethers.utils.parseEther("0.01")
    );
  });
});
```

3. **Griefing Attack Tests**:
   - Denial of service simulations
   - Fee-based attack economics
   - Resource exhaustion tests
   - Crowding-out attack tests

### Incentive Alignment Testing

Verifying economic security mechanisms:

1. **Slashing Mechanism Tests**:
   - Validator misbehavior simulations
   - Slashing condition verification
   - Punishment proportionality
   - Recovery from slashing events

2. **Example: Slashing Tests**:

```javascript
describe("Validator Slashing Mechanism", function() {
  let validatorSystem;
  let validators;
  let reporter;
  
  beforeEach(async function() {
    const accounts = await ethers.getSigners();
    validators = accounts.slice(1, 6);
    reporter = accounts[6];
    
    const ValidatorSystem = await ethers.getContractFactory("ValidatorSystem");
    validatorSystem = await ValidatorSystem.deploy();
    
    // Setup initial stakes for validators
    for (const validator of validators) {
      await validatorSystem.connect(validator).stake({
        value: ethers.utils.parseEther("32")
      });
    }
  });
  
  it("correctly slashes validators for double signing", async function() {
    const validator = validators[0];
    const blockHeight = 100;
    const proposal1 = ethers.utils.arrayify(ethers.utils.keccak256("0x123"));
    const proposal2 = ethers.utils.arrayify(ethers.utils.keccak256("0x456"));
    
    // Generate validator signatures
    const message1 = ethers.utils.solidityPack(
      ["uint256", "bytes32"],
      [blockHeight, ethers.utils.keccak256(proposal1)]
    );
    const message2 = ethers.utils.solidityPack(
      ["uint256", "bytes32"],
      [blockHeight, ethers.utils.keccak256(proposal2)]
    );
    
    const signature1 = await validator.signMessage(
      ethers.utils.arrayify(ethers.utils.keccak256(message1))
    );
    const signature2 = await validator.signMessage(
      ethers.utils.arrayify(ethers.utils.keccak256(message2))
    );
    
    // Check validator status before slashing
    expect(await validatorSystem.isActiveValidator(validator.address)).to.be.true;
    const initialStake = await validatorSystem.stakeOf(validator.address);
    expect(initialStake).to.equal(ethers.utils.parseEther("32"));
    
    // Report double signing
    await validatorSystem.connect(reporter).reportDoubleSigning(
      validator.address,
      blockHeight,
      proposal1,
      signature1,
      proposal2,
      signature2
    );
    
    // Verify validator was slashed
    expect(await validatorSystem.isActiveValidator(validator.address)).to.be.false;
    const finalStake = await validatorSystem.stakeOf(validator.address);
    
    // Should lose 50% of stake
    expect(finalStake).to.equal(initialStake.div(2));
    
    // Verify reporter got reward
    const reporterBalance = await validatorSystem.rewardPool(reporter.address);
    expect(reporterBalance).to.equal(initialStake.div(4)); // 25% of initial stake
  });
  
  it("verifies no false-positive slashing", async function() {
    const validator = validators[0];
    const blockHeight1 = 100;
    const blockHeight2 = 101;
    const proposal = ethers.utils.arrayify(ethers.utils.keccak256("0x123"));
    
    // Generate validator signatures for different blocks, which is fine
    const message1 = ethers.utils.solidityPack(
      ["uint256", "bytes32"],
      [blockHeight1, ethers.utils.keccak256(proposal)]
    );
    const message2 = ethers.utils.solidityPack(
      ["uint256", "bytes32"],
      [blockHeight2, ethers.utils.keccak256(proposal)]
    );
    
    const signature1 = await validator.signMessage(
      ethers.utils.arrayify(ethers.utils.keccak256(message1))
    );
    const signature2 = await validator.signMessage(
      ethers.utils.arrayify(ethers.utils.keccak256(message2))
    );
    
    // Attempt to report valid signing
    await expect(
      validatorSystem.connect(reporter).reportDoubleSigning(
        validator.address,
        blockHeight1,
        proposal,
        signature1,
        proposal,
        signature2
      )
    ).to.be.revertedWith("Not double signing");
    
    // Verify validator was not slashed
    expect(await validatorSystem.isActiveValidator(validator.address)).to.be.true;
    const stake = await validatorSystem.stakeOf(validator.address);
    expect(stake).to.equal(ethers.utils.parseEther("32"));
  });
  
  it("correctly handles stake withdrawal timeout after slashing", async function() {
    const validator = validators[0];
    const blockHeight = 100;
    const proposal1 = ethers.utils.arrayify(ethers.utils.keccak256("0x123"));
    const proposal2 = ethers.utils.arrayify(ethers.utils.keccak256("0x456"));
    
    // Generate signatures and report double signing
    // ... (similar to previous test)
    
    // Simulate reporter reporting and validator being slashed
    // (simplified for brevity)
    await validatorSystem.slashValidator(validator.address, 50);
    
    // Try to withdraw immediately
    await expect(
      validatorSystem.connect(validator).withdrawStake()
    ).to.be.revertedWith("Withdrawal locked after slashing");
    
    // Move forward past withdrawal lock period
    await ethers.provider.send("evm_increaseTime", [30 * 86400]); // 30 days
    await ethers.provider.send("evm_mine");
    
    // Should now be able to withdraw remaining stake
    await validatorSystem.connect(validator).withdrawStake();
    
    // Verify validator received remaining stake
    expect(await validatorSystem.stakeOf(validator.address)).to.equal(0);
  });
});
```

## Security Certification

### Testing Documentation

Creating audit-ready security documentation:

1. **Security Test Plan**:
   - Scope definition
   - Risk assessment
   - Test coverage goals
   - Testing methodologies

2. **Security Test Report Template**:

```markdown
# Security Test Report

## Executive Summary
[Brief summary of testing conducted and key findings]

## Test Scope
- Contracts tested: [list of contracts]
- Testing period: [dates]
- Testing methods: [static analysis, fuzzing, formal verification, etc.]
- Tools used: [Slither, Mythril, etc.]

## Risk Assessment

| Risk | Likelihood | Impact | Risk Score | Status |
|------|------------|--------|------------|--------|
| Reentrancy | Low | Critical | Medium | Mitigated |
| Access Control | Low | Critical | Medium | Mitigated |
| Oracle Manipulation | Medium | Critical | High | Mitigated |
| Integer Overflow | Low | High | Medium | Mitigated |
| DoS | Low | Medium | Low | Mitigated |

## Findings

### Critical Findings
[List of critical security issues found]

### High Findings
[List of high security issues found]

### Medium Findings
[List of medium security issues found]

### Low Findings
[List of low security issues found]

## Test Coverage Analysis
- Function coverage: XX%
- Branch coverage: XX% 
- Contract coverage: XX%

## Formal Verification Results
[Summary of formal verification findings]

## Economic Security Analysis
[Summary of economic attack vector analysis]

## Recommendations
[Prioritized list of security recommendations]
```

3. **Code Remediation Process**:
   - Issue triage workflow
   - Fix prioritization
   - Re-testing procedures
   - Security regression prevention

### Continuous Security Testing

Integrating security throughout the development lifecycle:

1. **Security CI Pipeline**:
   - Automated vulnerability scanning
   - Policy enforcement
   - Security gate configuration
   - Dashboard integration

2. **Example CI Security Setup**:

```yaml
# ci-security.yml
name: Security Testing

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * 0'  # Weekly run

jobs:
  static-analysis:
    name: Static Security Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.9'
      - name: Install Slither
        run: pip install slither-analyzer
      - name: Run Slither
        run: slither . --json slither-report.json
      - name: Process Slither Results
        run: python scripts/process_slither.py
      - name: Upload Slither Results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: slither-results.sarif
  
  mythril:
    name: Mythril Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Mythril
        uses: sailor0001/mythril-action@v2
        with:
          target: contracts/
  
  formal-verification:
    name: Formal Verification
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Certora CLI
        run: |
          curl -O https://certora.s3.amazonaws.com/certora-cli-v1.1.0-all.jar
          echo 'java -jar certora-cli-v1.1.0-all.jar "$@"' > certora
          chmod +x certora
      - name: Run Verification
        run: ./certora verify --spec specs/token.spec --contract Token --rule transferPreservesTotalSupply
  
  security-report:
    name: Generate Security Report
    runs-on: ubuntu-latest
    needs: [static-analysis, mythril, formal-verification]
    if: always()
    steps:
      - uses: actions/checkout@v3
      - name: Download All Artifacts
        uses: actions/download-artifact@v3
      - name: Combine Results
        run: node scripts/combine_security_results.js
      - name: Generate Security Report
        run: node scripts/generate_security_report.js
      - name: Upload Security Report
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: security-report.pdf
```

3. **Pre-deployment Security Checklist**:
   - Final security review steps
   - Risk assessment verification
   - Audit findings confirmation
   - Emergency response readiness

## Conclusion

Security testing is an essential part of blockchain application development, where vulnerabilities can lead to significant financial losses and damaged trust. By implementing a comprehensive security testing strategy that includes automated scanning, manual review, penetration testing, formal verification, and economic analysis, developers can identify and mitigate potential security risks before deployment.

The techniques and tools covered in this chapter provide a foundation for implementing effective security testing practices for ProzChain applications. Regular and thorough security testing helps ensure that blockchain applications are resilient against attacks and can maintain the trust of users and stakeholders.

## Next Steps

- [Property-Based Testing](./testing-framework-property-testing.md): Explore how property-based testing can complement security testing by identifying unexpected edge cases and vulnerabilities.
- [Mock Systems](./testing-framework-mock-systems.md): Learn how to create and use mock systems to simulate various security-critical scenarios.
- [Continuous Integration](./testing-framework-ci.md): Understand how to integrate security testing into your continuous integration pipeline.
- [Best Practices](./testing-framework-best-practices.md): Review best practices for maintaining security throughout the development lifecycle.
