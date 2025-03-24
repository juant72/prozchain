# Testing Tools

## Overview

Selecting the right testing tools is essential for effectively validating blockchain applications. This chapter explores the various testing frameworks, libraries, and utilities available for testing ProzChain applications. From development environments to specialized testing frameworks, these tools provide developers with the capabilities needed to thoroughly test their applications across different levels of the testing pyramid.

Understanding the strengths and weaknesses of each tool allows developers to choose the most appropriate solution for their specific testing needs, whether they're conducting unit tests, integration tests, security audits, or performance optimization.

## Hardhat

### Introduction to Hardhat

Hardhat is a comprehensive development environment for Ethereum and EVM-compatible chains:

```bash
# Create a new project
mkdir my-prozchain-project
cd my-prozchain-project
npm init -y

# Install Hardhat
npm install --save-dev hardhat

# Initialize Hardhat project
npx hardhat init

# Select "Create a JavaScript project"
```

Basic configuration:

```javascript
// hardhat.config.js
require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-ethers");

module.exports = {
  solidity: {
    version: "0.8.17",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      }
    }
  },
  networks: {
    hardhat: {
      chainId: 1337
    },
    localhost: {
      url: "http://127.0.0.1:8545"
    },
    prozchain: {
      url: process.env.PROZCHAIN_RPC_URL || "https://rpc.prozchain.network",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      chainId: 246
    }
  }
};
```

### Hardhat Testing Capabilities

Hardhat offers powerful features specifically for blockchain testing:

```javascript
// test/Token.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Token Contract", function() {
  let Token;
  let token;
  let owner;
  let addr1;
  let addr2;
  let addrs;

  beforeEach(async function() {
    Token = await ethers.getContractFactory("Token");
    [owner, addr1, addr2, ...addrs] = await ethers.getSigners();

    token = await Token.deploy("Test Token", "TST", 1000000);
    await token.deployed();
  });

  describe("Deployment", function() {
    it("Should set the right owner", async function() {
      expect(await token.owner()).to.equal(owner.address);
    });

    it("Should assign the total supply of tokens to the owner", async function() {
      const ownerBalance = await token.balanceOf(owner.address);
      expect(await token.totalSupply()).to.equal(ownerBalance);
    });
  });

  describe("Transactions", function() {
    it("Should transfer tokens between accounts", async function() {
      // Transfer 50 tokens from owner to addr1
      await token.transfer(addr1.address, 50);
      const addr1Balance = await token.balanceOf(addr1.address);
      expect(addr1Balance).to.equal(50);

      // Transfer 50 tokens from addr1 to addr2
      await token.connect(addr1).transfer(addr2.address, 50);
      const addr2Balance = await token.balanceOf(addr2.address);
      expect(addr2Balance).to.equal(50);
    });

    it("Should fail if sender doesn't have enough tokens", async function() {
      const initialOwnerBalance = await token.balanceOf(owner.address);
      
      // Try to send 1 token from addr1 (0 tokens) to owner
      await expect(
        token.connect(addr1).transfer(owner.address, 1)
      ).to.be.revertedWith("Not enough tokens");

      // Owner balance shouldn't change
      expect(await token.balanceOf(owner.address)).to.equal(initialOwnerBalance);
    });
  });
});
```

Key Hardhat testing features include:

1. **Console Debugging**: The `console.log()` function can be used directly within Solidity code.
2. **Stack Traces**: Full JavaScript stack traces for easier debugging.
3. **Network Management**: Built-in local Ethereum network for testing.
4. **Test Helpers**: Utilities for simulating various blockchain conditions.

### Hardhat Plugins

Essential plugins that extend Hardhat's testing capabilities:

```bash
# Install multiple plugins
npm install --save-dev @nomiclabs/hardhat-waffle @nomiclabs/hardhat-ethers @nomiclabs/hardhat-etherscan solidity-coverage hardhat-gas-reporter
```

Plugin configuration:

```javascript
// hardhat.config.js
require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-ethers");
require("@nomiclabs/hardhat-etherscan");
require("solidity-coverage");
require("hardhat-gas-reporter");

module.exports = {
  // ... other config
  gasReporter: {
    enabled: process.env.REPORT_GAS ? true : false,
    currency: 'USD',
    gasPrice: 21,
    coinmarketcap: process.env.COINMARKETCAP_API_KEY,
    excludeContracts: ['mocks/'],
    src: './contracts',
  },
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY
  }
};
```

Key Plugins:

1. **hardhat-waffle**: Testing with Ethereum-Waffle framework
2. **hardhat-ethers**: Integration with ethers.js
3. **solidity-coverage**: Code coverage analysis
4. **hardhat-gas-reporter**: Gas usage reports

Running coverage tests:

```bash
npx hardhat coverage
```

## Foundry

### Introduction to Foundry

Foundry is a fast, portable and modular toolkit for Ethereum application development:

```bash
# Using foundryup
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Initialize a new project
forge init my-foundry-project
cd my-foundry-project
```

Foundry project structure:

```
my-foundry-project/
├── lib/                # Dependencies
├── src/                # Contract source files
├── test/               # Test files
├── script/             # Deployment scripts
├── foundry.toml        # Configuration
└── remappings.txt      # Import remappings
```

Basic configuration:

```toml
# foundry.toml
[profile.default]
src = "src"
test = "test"
script = "script"
out = "out"
libs = ["lib"]
solc_version = "0.8.17"
optimizer = true
optimizer_runs = 200

[profile.ci]
fuzz_runs = 1000
verbosity = 4
```

### Foundry Testing Features

Foundry allows writing tests directly in Solidity:

```solidity
// test/Token.t.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/Token.sol";

contract TokenTest is Test {
    Token token;
    address alice = address(1);
    address bob = address(2);

    function setUp() public {
        token = new Token("Test Token", "TST", 1000000);
        // Give alice some tokens
        token.transfer(alice, 1000);
    }

    function testTransfer() public {
        // Simulate alice transferring to bob
        vm.prank(alice);
        token.transfer(bob, 500);

        // Check balances
        assertEq(token.balanceOf(alice), 500);
        assertEq(token.balanceOf(bob), 500);
    }

    function testFailInsufficientBalance() public {
        // Attempt to transfer more tokens than available
        vm.prank(alice);
        token.transfer(bob, 1500); // Should fail
    }
}
```

Key Foundry testing features include:

1. **Cheatcodes**: Special functions provided by the `vm` object to manipulate blockchain state
2. **Fast Execution**: Tests run directly against the EVM, making them faster than JavaScript-based tests
3. **Fuzzing**: Automatically generates random inputs for property-based testing
4. **Symbolic Execution**: Can detect edge cases in contract logic

### Fuzzing with Foundry

Property-based testing with random inputs:

```solidity
// test/TokenFuzz.t.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/Token.sol";

contract TokenFuzzTest is Test {
    Token token;
    address alice = address(1);
    address bob = address(2);

    function setUp() public {
        token = new Token("Test Token", "TST", 1000000);
        // Give alice some tokens
        token.transfer(alice, 1000);
    }

    // Fuzz test with random amounts
    function testFuzzTransfer(uint256 amount) public {
        // Constrain the amount to be valid (not more than alice has)
        vm.assume(amount <= 1000);
        
        // Store original balances
        uint256 aliceInitialBalance = token.balanceOf(alice);
        uint256 bobInitialBalance = token.balanceOf(bob);
        
        // Simulate alice transferring to bob
        vm.prank(alice);
        token.transfer(bob, amount);
        
        // Check balances
        assertEq(token.balanceOf(alice), aliceInitialBalance - amount);
        assertEq(token.balanceOf(bob), bobInitialBalance + amount);
    }
}
```

## Truffle

### Introduction to Truffle

Truffle is one of the original Ethereum development frameworks:

```bash
# Install Truffle globally
npm install -g truffle

# Initialize a new project
mkdir my-truffle-project
cd my-truffle-project
truffle init
```

Project structure:

```
my-truffle-project/
├── contracts/              # Smart contract source files
├── migrations/             # Deployment scripts
├── test/                   # Test files
└── truffle-config.js       # Configuration file
```

Basic configuration:

```javascript
// truffle-config.js
module.exports = {
  networks: {
    development: {
      host: "127.0.0.1",
      port: 7545,
      network_id: "*"
    },
    prozchain: {
      host: "rpc.prozchain.network",
      port: 8545,
      network_id: "246",
      from: process.env.DEPLOYER_ADDRESS,
      gas: 5000000,
      gasPrice: 20000000000
    }
  },
  compilers: {
    solc: {
      version: "0.8.17",
      settings: {
        optimizer: {
          enabled: true,
          runs: 200
        }
      }
    }
  }
};
```

### Truffle Testing Capabilities

Testing with Truffle:

```javascript
// test/token.test.js
const Token = artifacts.require("Token");
const { BN, expectEvent, expectRevert } = require('@openzeppelin/test-helpers');

contract("Token", function(accounts) {
  const [owner, alice, bob] = accounts;
  let token;

  beforeEach(async function() {
    token = await Token.new("Test Token", "TST", 1000000, { from: owner });
  });

  describe("Basic Functionality", function() {
    it("should return the correct name and symbol", async function() {
      const name = await token.name();
      const symbol = await token.symbol();
      
      assert.equal(name, "Test Token");
      assert.equal(symbol, "TST");
    });

    it("should transfer tokens correctly", async function() {
      // Transfer to alice
      const amount = new BN('500');
      const receipt = await token.transfer(alice, amount, { from: owner });
      
      // Check event
      expectEvent(receipt, 'Transfer', {
        from: owner,
        to: alice,
        value: amount
      });
      
      // Check balances
      const aliceBalance = await token.balanceOf(alice);
      assert.equal(aliceBalance.toString(), '500');
    });

    it("should revert when transferring more than balance", async function() {
      // Try to transfer more than balance
      await expectRevert(
        token.transfer(owner, 100, { from: alice }),
        "Not enough tokens"
      );
    });
  });
});
```

Key features:

1. **Contract Abstractions**: Easy interaction with deployed contracts
2. **Migrations**: Structured deployment framework
3. **Integration with Ganache**: Local blockchain for testing
4. **Plugin System**: Extensions for specific needs

## Specialized Testing Tools

### Slither

Slither is a static analysis framework for Solidity smart contracts:

```bash
# Install Slither
pip3 install slither-analyzer

# Run Slither on contracts
slither contracts/
```

Example output:

```
INFO:Detectors:
Pragma version^0.8.17 (contracts/Token.sol#2) allows old versions
Reference: https://github.com/crytic/slither/wiki/Detector-Documentation#incorrect-versions-of-solidity

INFO:Detectors:
Token.transfer(address,uint256) (contracts/Token.sol#37-44) uses a dangerous strict equality:
        - require(bool)(balances[msg.sender] >= amount) (contracts/Token.sol#38)
Reference: https://github.com/crytic/slither/wiki/Detector-Documentation#dangerous-strict-equalities
```

Key features:

1. **Vulnerability Detection**: Identifies common security issues
2. **Contract Visualization**: Generates control flow graphs
3. **Inheritance Analysis**: Maps contract relationships

### Echidna

Fuzzing-based smart contract security tool:

```bash
# Install Echidna
docker pull trailofbits/echidna

# Run Echidna tests
docker run -v "$PWD":/src -it trailofbits/echidna echidna-test /src/contracts/TestToken.sol
```

Example test contract:

```solidity
// contracts/EchidnaTestToken.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "./Token.sol";

contract EchidnaTestToken is Token {
    constructor() Token("Test Token", "TST", 10000) {}
    
    // This should always be true - total supply is constant
    function echidna_total_supply_constant() public view returns (bool) {
        return totalSupply() == 10000;
    }
    
    // This should always be true - sum of all balances equals total supply
    function echidna_balance_sum_equals_total() public view returns (bool) {
        return balanceOf(msg.sender) <= totalSupply();
    }
}
```

Key features:

1. **Property-based Testing**: Verify invariants hold across many inputs
2. **Coverage-guided Fuzzing**: Focuses on code paths not yet explored
3. **Integration with CI**: Automate security testing

### Mythril

Symbolic execution tool for smart contract security analysis:

```bash
# Install Mythril
pip3 install mythril

# Analyze a contract
myth analyze contracts/Token.sol
```

Example output:

```
==== Integer Arithmetic Bugs ====
SWC ID: 101
Severity: High
Contract: Token
Function name: transfer(address,uint256)
PC address: 1234
Estimated Gas Usage: 1082 - 1577
A possible integer overflow occurs where the addition might result in a value higher than the maximum value of the type.
--------------------
In file: contracts/Token.sol:41

balances[to] += amount

--------------------
```

Key features:

1. **Symbolic Execution**: Discovers difficult-to-find vulnerabilities
2. **Automated Testing**: Helps identify security issues early
3. **Structured Output**: Detailed reports with mitigation suggestions

### Brownie

Python-based smart contract development and testing framework:

```bash
# Install Brownie
pip install eth-brownie

# Initialize a new project
brownie init
```

Example test script:

```python
# tests/test_token.py
import pytest
from brownie import Token, accounts, exceptions

@pytest.fixture
def token():
    return accounts[0].deploy(Token, "Test Token", "TST", 1000000)

def test_initial_supply(token):
    assert token.totalSupply() == 1000000

def test_transfer(token):
    # Initial balances
    assert token.balanceOf(accounts[0]) == 1000000
    assert token.balanceOf(accounts[1]) == 0
    
    # Transfer tokens
    token.transfer(accounts[1], 500, {'from': accounts[0]})
    
    # Check balances after transfer
    assert token.balanceOf(accounts[0]) == 999500
    assert token.balanceOf(accounts[1]) == 500

def test_insufficient_balance(token):
    # Try to transfer more tokens than balance
    with pytest.raises(exceptions.VirtualMachineError):
        token.transfer(accounts[0], 100, {'from': accounts[1]})
```

Key features:

1. **Python-based Testing**: More accessible for Python developers
2. **Console Access**: Interactive debugging features
3. **Network Management**: Multiple deployment environment support
4. **Gas Profiling**: Built-in gas usage tracking

## Testing Interoperability

### Cross-Framework Testing

Using multiple frameworks together for comprehensive testing:

```javascript
// Using Hardhat to generate test data for Foundry
// scripts/generate-test-data.js
const fs = require('fs');
const path = require('path');
const { ethers } = require('hardhat');

async function main() {
  const [deployer] = await ethers.getSigners();

  // Deploy test token
  const Token = await ethers.getContractFactory("Token");
  const token = await Token.deploy("Test Token", "TST", 1000000);
  await token.deployed();
  
  // Generate test transactions
  await token.transfer(ethers.Wallet.createRandom().address, 50000);
  await token.transfer(ethers.Wallet.createRandom().address, 30000);
  await token.transfer(ethers.Wallet.createRandom().address, 20000);
  
  // Save contract address for Foundry tests
  const testData = {
    tokenAddress: token.address,
    deployerAddress: deployer.address,
    initialSupply: 1000000,
    networkId: (await ethers.provider.getNetwork()).chainId
  };
  
  // Write to a file that Foundry tests can read
  fs.writeFileSync(
    path.join(__dirname, '../test-data.json'),
    JSON.stringify(testData, null, 2)
  );
  
  console.log("Test data generated:", testData);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
```

Corresponding Foundry test:

```solidity
// test/InteropToken.t.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/Token.sol";

interface IToken {
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
}

contract InteropTokenTest is Test {
    IToken token;
    address deployerAddress;
    
    function setUp() public {
        // Load test data generated by Hardhat
        string memory json = vm.readFile("test-data.json");
        address tokenAddress = abi.decode(vm.parseJson(json, ".tokenAddress"), (address));
        deployerAddress = abi.decode(vm.parseJson(json, ".deployerAddress"), (address));
        
        // Connect to the deployed token
        token = IToken(tokenAddress);
    }
    
    function testDeployedToken() public {
        // Test token deployed by Hardhat
        assertEq(token.name(), "Test Token");
        assertEq(token.symbol(), "TST");
        assertEq(token.totalSupply(), 1000000);
        
        // Check that transfers happened
        uint256 deployerBalance = token.balanceOf(deployerAddress);
        assertLt(deployerBalance, 1000000);
    }
}
```

### Testing Between Local and Live Networks

Running tests against both local and test networks:

```javascript
// test/multi-environment.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Cross-Environment Testing", function() {
  let token;
  
  before(async function() {
    // Check if we're on a live network
    const networkName = hre.network.name;
    
    if (networkName === "hardhat" || networkName === "localhost") {
      // On local network - deploy fresh contract
      const Token = await ethers.getContractFactory("Token");
      token = await Token.deploy("Test Token", "TST", 1000000);
      await token.deployed();
    } else {
      // On test network - use existing deployment
      const tokenAddress = process.env.TOKEN_ADDRESS;
      if (!tokenAddress) {
        this.skip(); // Skip test if no address provided
      }
      token = await ethers.getContractAt("Token", tokenAddress);
    }
  });
  
  it("has correct token details", async function() {
    expect(await token.name()).to.equal("Test Token");
    expect(await token.symbol()).to.equal("TST");
    expect(await token.totalSupply()).to.be.gt(0);
  });
  
  it("allows transfers between accounts", async function() {
    // This test only runs on local networks where we have control
    if (hre.network.name !== "hardhat" && hre.network.name !== "localhost") {
      this.skip();
    }
    
    const [owner, recipient] = await ethers.getSigners();
    
    // Initial balances
    const initialOwnerBalance = await token.balanceOf(owner.address);
    const initialRecipientBalance = await token.balanceOf(recipient.address);
    
    // Transfer amount
    const transferAmount = 100;
    
    // Transfer tokens
    await token.transfer(recipient.address, transferAmount);
    
    // Check balances after transfer
    expect(await token.balanceOf(owner.address)).to.equal(initialOwnerBalance.sub(transferAmount));
    expect(await token.balanceOf(recipient.address)).to.equal(initialRecipientBalance.add(transferAmount));
  });
});
```

## CI/CD Integration

### GitHub Actions Integration

Setting up GitHub Actions for automated testing:

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  hardhat-tests:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '16'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run Hardhat tests
      run: npx hardhat test
    
    - name: Generate coverage report
      run: npx hardhat coverage
    
    - name: Upload coverage reports
      uses: codecov/codecov-action@v3
      with:
        directory: ./coverage
  
  foundry-tests:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
      with:
        submodules: recursive
    
    - name: Install Foundry
      uses: foundry-rs/foundry-toolchain@v1
    
    - name: Run Forge tests
      run: forge test -vvv
    
    - name: Run Forge coverage
      run: forge coverage --report lcov
    
    - name: Upload coverage reports
      uses: codecov/codecov-action@v3
      with:
        files: ./lcov.info
  
  security-analysis:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.10'
    
    - name: Install Slither
      run: pip install slither-analyzer
    
    - name: Run Slither
      run: slither . --json slither-report.json || true
    
    - name: Upload Slither report
      uses: actions/upload-artifact@v3
      with:
        name: slither-report
        path: slither-report.json
```

### Testing Matrix Strategy

Testing across multiple configurations:

```yaml
jobs:
  multi-environment-tests:
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        node: [14, 16, 18]
        solidity: ['0.8.15', '0.8.17']
        include:
          - node: 18
            solidity: '0.8.17'
            coverage: true
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js ${{ matrix.node }}
      uses: actions/setup-node@v3
      with:
        node-version: ${{ matrix.node }}
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Update Solidity version
      run: |
        sed -i 's/solidity: "0.8.[0-9][0-9]"/solidity: "${{ matrix.solidity }}"/g' hardhat.config.js
    
    - name: Run Hardhat tests
      run: npx hardhat test
    
    - name: Generate coverage report
      if: ${{ matrix.coverage }}
      run: npx hardhat coverage
```

## Tool Selection Guide

### Choosing the Right Testing Framework

Considerations for selecting the appropriate testing tools:

1. **Project Requirements**:
   * Contract complexity
   * Team expertise
   * Integration needs
   * Testing requirements

2. **Framework Comparison**:

   | Framework | Language      | Speed | Features                           | Best For                                    |
   |-----------|---------------|-------|------------------------------------|--------------------------------------------|
   | Hardhat   | JavaScript/TS | Good  | Full development environment       | Full-stack developers, complex applications |
   | Foundry   | Solidity      | Fast  | Fuzzing, symbolic execution        | Smart contract developers, security focus   |
   | Truffle   | JavaScript    | Good  | Structured project management      | Beginners, traditional workflows            |
   | Brownie   | Python        | Good  | Data analysis, scientific testing  | Python developers, data-intensive testing   |

3. **Tool Integration Guidelines**:
   * Use Hardhat or Truffle for JavaScript/TypeScript frontends
   * Use Foundry for security-focused testing
   * Use specialized tools (Slither, Echidna) for security audits
   * Consider combining tools for comprehensive test coverage

### Example Testing Strategy

A comprehensive testing strategy might combine multiple tools:

1. **Development Workflow**:
   * Hardhat for daily development and unit testing
   * Foundry for fuzzing and invariant testing
   * Slither for static analysis during code review
   * Mythril for periodic deep security analysis

2. **CI/CD Pipeline**:
   * Run Hardhat tests on every commit
   * Run Foundry tests on every PR
   * Run security tools on merge to main branch
   * Generate and publish coverage reports

## Conclusion

The blockchain testing ecosystem offers a variety of tools, each with its own strengths and weaknesses. By understanding these tools and choosing the right ones for your specific needs, you can build a comprehensive testing strategy that ensures your ProzChain applications are secure, reliable, and robust.

Key takeaways:

1. **Diversify Testing Tools**: Different tools catch different issues
2. **Automate Testing**: Integrate testing into your CI/CD pipeline
3. **Balance Coverage**: Use a mix of unit, integration, and security tests
4. **Stay Updated**: The blockchain testing ecosystem evolves rapidly

The next chapter will explore continuous integration strategies for blockchain applications, building on the testing tools covered here.
