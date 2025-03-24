# CI for Different Blockchain Networks

## Overview

Blockchain applications often need to be tested across multiple networks, from local development environments to public testnets and production networks. This chapter explores strategies for configuring CI pipelines to test ProzChain applications across different blockchain environments.

## Testing on Public Networks

### Testnet Configuration

Setting up CI to test on public testnets:

```yaml
jobs:
  testnet-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        network: [sepolia, goerli, prozchain-testnet]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Get Test Account
        id: account
        uses: getsentry/action-github-app-token@v1
        with:
          app_id: ${{ secrets.FAUCET_APP_ID }}
          private_key: ${{ secrets.FAUCET_PRIVATE_KEY }}
      
      - name: Fund Test Account
        run: node scripts/fund-testnet-account.js
        env:
          NETWORK: ${{ matrix.network }}
          ACCOUNT_PRIVATE_KEY: ${{ steps.account.outputs.token }}
      
      - name: Run Tests on ${{ matrix.network }}
        run: npm run test:integration
        env:
          HARDHAT_NETWORK: ${{ matrix.network }}
          PRIVATE_KEY: ${{ steps.account.outputs.token }}
```

Example script to fund test accounts:

```javascript
// scripts/fund-testnet-account.js
const { ethers } = require("hardhat");
const axios = require("axios");

async function main() {
  const network = process.env.NETWORK;
  const privateKey = process.env.ACCOUNT_PRIVATE_KEY;
  
  if (!privateKey) {
    throw new Error("No private key provided");
  }
  
  const wallet = new ethers.Wallet(privateKey, ethers.provider);
  const address = wallet.address;
  
  console.log(`Funding test account ${address} on ${network}...`);
  
  // Request funds from appropriate faucet
  let faucetUrl;
  let faucetResponse;
  
  switch (network) {
    case 'sepolia':
      faucetUrl = `https://sepoliafaucet.com/api/fund?address=${address}`;
      faucetResponse = await axios.post(faucetUrl, {}, {
        headers: { 'Authorization': `Bearer ${process.env.ALCHEMY_API_KEY}` }
      });
      break;
      
    case 'goerli':
      faucetUrl = `https://goerlifaucet.com/api/fund?address=${address}`;
      faucetResponse = await axios.post(faucetUrl, {}, {
        headers: { 'Authorization': `Bearer ${process.env.ALCHEMY_API_KEY}` }
      });
      break;
      
    case 'prozchain-testnet':
      faucetUrl = `https://faucet.prozchain.network/api/fund?address=${address}`;
      faucetResponse = await axios.post(faucetUrl);
      break;
      
    default:
      throw new Error(`Unsupported network: ${network}`);
  }
  
  console.log(`Faucet response: ${JSON.stringify(faucetResponse.data)}`);
  
  // Wait for funding transaction to confirm
  console.log("Waiting for transaction confirmation...");
  await new Promise(resolve => setTimeout(resolve, 20000)); // Wait 20 seconds
  
  // Check balance
  const balance = await ethers.provider.getBalance(address);
  console.log(`Account balance: ${ethers.utils.formatEther(balance)} ETH`);
  
  if (balance.eq(0)) {
    throw new Error("Failed to fund account");
  }
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
```

### Managing Test Accounts

Securing and managing accounts for CI testing:

```javascript
// scripts/generate-test-accounts.js
const { ethers } = require('ethers');
const fs = require('fs');
const path = require('path');

// Generate deterministic accounts for testing
function generateTestAccounts() {
  // Use a fixed mnemonic for deterministic test accounts
  const mnemonic = process.env.TEST_MNEMONIC || 
    'test test test test test test test test test test test junk';
  
  const hdNode = ethers.utils.HDNode.fromMnemonic(mnemonic);
  const accounts = [];
  
  // Generate 10 accounts
  for (let i = 0; i < 10; i++) {
    const childNode = hdNode.derivePath(`m/44'/60'/0'/0/${i}`);
    accounts.push({
      address: childNode.address,
      privateKey: childNode.privateKey
    });
  }
  
  return accounts;
}

// Save accounts to file
function saveAccounts(accounts) {
  const outputDir = path.join(__dirname, '../test-accounts');
  
  // Create directory if it doesn't exist
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  
  // Save accounts JSON
  const outputPath = path.join(outputDir, 'accounts.json');
  fs.writeFileSync(outputPath, JSON.stringify(accounts, null, 2));
  
  console.log(`Generated ${accounts.length} test accounts at ${outputPath}`);
  
  // Create Hardhat config fragment
  const hardhatAccountsPath = path.join(outputDir, 'hardhat-accounts.js');
  const privateKeys = accounts.map(account => account.privateKey);
  
  fs.writeFileSync(
    hardhatAccountsPath,
    `module.exports = ${JSON.stringify(privateKeys, null, 2)};`
  );
}

// Execute if run directly
if (require.main === module) {
  const accounts = generateTestAccounts();
  saveAccounts(accounts);
}

module.exports = { generateTestAccounts };
```

### Forking Mainnet for Tests

Using mainnet forking for realistic testing:

```yaml
jobs:
  mainnet-fork-tests:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Start Hardhat Node with Mainnet Fork
        run: |
          npx hardhat node \
            --fork https://eth-mainnet.alchemyapi.io/v2/${{ secrets.ALCHEMY_KEY }} \
            --fork-block-number 15000000 \
            --no-deploy &
          sleep 5
      
      - name: Run Fork Tests
        run: npm run test:fork
        env:
          HARDHAT_NETWORK: localhost
          FORK_ENABLED: "true"
          FORK_BLOCK_NUMBER: "15000000"
```

## Handling Multiple Chain Tests

### Cross-Chain Test Configuration

Setting up tests across multiple blockchains:

```javascript
// hardhat.config.js
require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-ethers");

const accounts = process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [];

module.exports = {
  solidity: "0.8.17",
  networks: {
    hardhat: {
      forking: process.env.FORK_ENABLED === "true" ? {
        url: process.env.FORK_URL || `https://eth-mainnet.alchemyapi.io/v2/${process.env.ALCHEMY_KEY}`,
        blockNumber: process.env.FORK_BLOCK_NUMBER ? parseInt(process.env.FORK_BLOCK_NUMBER) : undefined,
      } : undefined,
      chainId: 1337,
    },
    // Ethereum networks
    mainnet: {
      url: `https://eth-mainnet.alchemyapi.io/v2/${process.env.ALCHEMY_KEY}`,
      accounts,
    },
    sepolia: {
      url: `https://eth-sepolia.g.alchemy.com/v2/${process.env.ALCHEMY_KEY}`,
      accounts,
    },
    goerli: {
      url: `https://eth-goerli.alchemyapi.io/v2/${process.env.ALCHEMY_KEY}`,
      accounts,
    },
    // ProzChain networks
    "prozchain-mainnet": {
      url: process.env.PROZCHAIN_MAINNET_URL || "https://rpc.prozchain.network",
      accounts,
      chainId: 246,
    },
    "prozchain-testnet": {
      url: process.env.PROZCHAIN_TESTNET_URL || "https://testnet-rpc.prozchain.network",
      accounts,
      chainId: 24601,
    },
    // Other networks
    polygon: {
      url: `https://polygon-mainnet.g.alchemy.com/v2/${process.env.ALCHEMY_KEY}`,
      accounts,
      chainId: 137,
    },
    arbitrum: {
      url: `https://arb-mainnet.g.alchemy.com/v2/${process.env.ALCHEMY_KEY}`,
      accounts,
      chainId: 42161,
    },
  },
  mocha: {
    timeout: 60000
  },
};
```

### Chain-Specific Test Helpers

Adapting tests for different blockchain environments:

```javascript
// test/helpers/networkSpecificTests.js
const { network } = require("hardhat");

// Skip tests that aren't applicable on current network
function skipUnlessNetwork(networkNames) {
  const networks = Array.isArray(networkNames) ? networkNames : [networkNames];
  const currentNetwork = network.name;
  
  return function() {
    if (!networks.includes(currentNetwork)) {
      this.skip();
    }
  };
}

// Run network-specific before/after hooks
function setupNetworkHooks() {
  before(async function() {
    // Common setup for all networks
    
    // Network-specific setup
    switch (network.name) {
      case 'hardhat':
        // Local development setup
        break;
        
      case 'prozchain-testnet':
        // ProzChain specific setup
        this.timeout(120000); // Longer timeout for ProzChain
        break;
        
      case 'sepolia':
      case 'goerli':
        // Ethereum testnet setup
        this.timeout(90000); // Medium timeout
        break;
    }
  });
  
  after(async function() {
    // Network-specific cleanup
    switch (network.name) {
      case 'hardhat':
        // Nothing needed for local
        break;
        
      default:
        // Allow time for transactions to confirm on real networks
        await new Promise(resolve => setTimeout(resolve, 5000));
        break;
    }
  });
}

module.exports = {
  skipUnlessNetwork,
  setupNetworkHooks
};
```

### Network-Specific Job Matrices

Dynamically configuring CI jobs based on network requirements:

```yaml
jobs:
  network-matrix:
    name: Generate Network Matrix
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.set-matrix.outputs.matrix }}
      
    steps:
      - id: set-matrix
        run: |
          # Create dynamic matrix based on branch/event
          if [[ "${{ github.event_name }}" == "schedule" || "${{ github.ref }}" == "refs/heads/main" ]]; then
            # Full test on schedule or main branch
            echo "matrix={\"network\":[\"hardhat\",\"prozchain-testnet\",\"ethereum-goerli\",\"polygon-mumbai\"]}" >> $GITHUB_OUTPUT
          elif [[ "${{ github.event_name }}" == "pull_request" ]]; then
            # Reduced tests for PRs
            echo "matrix={\"network\":[\"hardhat\"]}" >> $GITHUB_OUTPUT
          else
            # Default test set
            echo "matrix={\"network\":[\"hardhat\",\"prozchain-testnet\"]}" >> $GITHUB_OUTPUT
          fi
  
  run-tests:
    needs: network-matrix
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{fromJson(needs.network-matrix.outputs.matrix)}}
      fail-fast: false
    
    steps:
      - uses: actions/checkout@v3
      # ... test steps with matrix.network
```

## Network-Specific Test Features

### Test Data Seeding

Preparing test environments on different networks:

```javascript
// scripts/seed-test-data.js
const { ethers } = require("hardhat");

async function main() {
  const network = hre.network.name;
  const [deployer] = await ethers.getSigners();
  
  console.log("Seeding test data on network:", network);
  console.log("Account balance:", ethers.utils.formatEther(await deployer.getBalance()));
  
  // Deploy test tokens
  const Token = await ethers.getContractFactory("TestToken");
  const token = await Token.deploy("Test Token", "TST");
  await token.deployed();
  
  console.log(`Test token deployed: ${token.address}`);
  
  // Mint test tokens based on network
  let mintAmount;
  switch(network) {
    case 'hardhat':
      mintAmount = ethers.utils.parseEther("10000000");
      break;
    case 'prozchain-testnet':
      mintAmount = ethers.utils.parseEther("1000000");
      break;
    default:
      mintAmount = ethers.utils.parseEther("100000");
  }
  
  await token.mint(deployer.address, mintAmount);
  
  // Deploy test contracts
  const TestSystem = await ethers.getContractFactory("TestSystem");
  const system = await TestSystem.deploy(token.address);
  await system.deployed();
  
  console.log(`Test system deployed: ${system.address}`);
  
  // Save deployment info for tests
  const fs = require('fs');
  const path = require('path');
  const deploymentPath = path.join(__dirname, '..', 'deployments', network);
  
  if (!fs.existsSync(deploymentPath)) {
    fs.mkdirSync(deploymentPath, { recursive: true });
  }
  
  fs.writeFileSync(
    path.join(deploymentPath, 'testContracts.json'),
    JSON.stringify({
      token: token.address,
      system: system.address,
      deployer: deployer.address,
      blockNumber: await ethers.provider.getBlockNumber()
    }, null, 2)
  );
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
```

### Testing Network Properties

Verifying blockchain-specific behaviors:

```javascript
// test/network-specific/chainProperties.test.js
const { expect } = require("chai");
const { ethers } = require("hardhat");
const { skipUnlessNetwork } = require("../helpers/networkSpecificTests");

describe("Network Specific Properties", function() {
  let provider;
  let accounts;
  
  before(async function() {
    provider = ethers.provider;
    accounts = await ethers.getSigners();
  });
  
  describe("ProzChain Specific Features", function() {
    beforeEach(skipUnlessNetwork(['prozchain-testnet', 'prozchain-mainnet']));
    
    it("verifies expected block time", async function() {
      // Get two consecutive blocks
      const blockNumber = await provider.getBlockNumber();
      const block1 = await provider.getBlock(blockNumber - 5);
      const block2 = await provider.getBlock(blockNumber);
      
      // Calculate average block time
      const timeDiff = block2.timestamp - block1.timestamp;
      const avgBlockTime = timeDiff / 5;
      
      // ProzChain has ~3 second block times
      expect(avgBlockTime).to.be.closeTo(3, 1);
    });
    
    it("handles ProzChain-specific transaction types", async function() {
      // Test ProzChain-specific transaction features if applicable
      // This is a placeholder for network-specific tests
      this.skip(); // Remove this line when implementing actual tests
    });
  });
  
  describe("Ethereum Network Features", function() {
    beforeEach(skipUnlessNetwork(['ethereum-mainnet', 'goerli', 'sepolia']));
    
    it("follows EIP-1559 fee market", async function() {
      // Check if the network supports EIP-1559
      const block = await provider.getBlock("latest");
      
      // EIP-1559 blocks have a baseFeePerGas property
      expect(block).to.have.property('baseFeePerGas');
      expect(block.baseFeePerGas).to.be.gt(0);
    });
  });
});
```

## Conclusion

Configuring CI for different blockchain networks requires careful planning and specialized strategies. By adopting network-specific configurations, test helpers, and deployment approaches, teams can ensure their applications are thoroughly tested across all target environments.

The techniques covered in this chapter help ensure that applications work consistently across different blockchain networks, detecting environment-specific issues early in the development process. In the next chapter, we'll explore security considerations for CI pipelines.
