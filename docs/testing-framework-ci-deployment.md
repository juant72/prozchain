# Continuous Deployment

## Overview

Continuous Deployment (CD) builds upon CI by automating the process of delivering code changes to production environments after passing all tests. In blockchain contexts, CD requires special considerations due to the immutable nature of deployed smart contracts and the need for rigorous verification before any deployment to production networks.

This chapter explores strategies for implementing safe, reliable continuous deployment pipelines for ProzChain applications across different blockchain networks.

## Automated Release Process

### Release Pipeline Architecture

The core components of a blockchain CD pipeline:

```yaml
name: Continuous Deployment

on:
  push:
    tags:
      - 'v*'

jobs:
  prepare:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.get_version.outputs.version }}
      
    steps:
      - uses: actions/checkout@v3
      
      - name: Get Version
        id: get_version
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          echo "Releasing version $VERSION"
  
  test:
    needs: prepare
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Comprehensive Tests
        run: npm run test:all
      
      - name: Run Security Audit
        run: npm run audit:prod
  
  build:
    needs: [prepare, test]
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Build Artifacts
        run: npm run build
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: build-artifacts
          path: |
            artifacts/
            dist/
  
  deploy-testnet:
    needs: [prepare, build]
    runs-on: ubuntu-latest
    environment: testnet
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Download Artifacts
        uses: actions/download-artifact@v3
        with:
          name: build-artifacts
      
      - name: Deploy to Testnet
        run: npx hardhat deploy --network testnet
        env:
          PRIVATE_KEY: ${{ secrets.TESTNET_DEPLOYER_KEY }}
      
      - name: Verify Contracts
        run: npx hardhat verify --network testnet
        env:
          ETHERSCAN_API_KEY: ${{ secrets.ETHERSCAN_API_KEY }}
  
  approve-mainnet:
    needs: deploy-testnet
    runs-on: ubuntu-latest
    environment: 
      name: mainnet
      url: https://etherscan.io/address/${{ steps.get_addresses.outputs.contract_address }}
    
    steps:
      - name: Get Contract Addresses
        id: get_addresses
        run: echo "contract_address=0x1234...5678" >> $GITHUB_OUTPUT
      
      # This job requires manual approval through GitHub environments
  
  deploy-mainnet:
    needs: approve-mainnet
    runs-on: ubuntu-latest
    environment: mainnet
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Download Artifacts
        uses: actions/download-artifact@v3
        with:
          name: build-artifacts
      
      - name: Deploy to Mainnet
        run: npx hardhat deploy --network mainnet
        env:
          PRIVATE_KEY: ${{ secrets.MAINNET_DEPLOYER_KEY }}
      
      - name: Verify Contracts
        run: npx hardhat verify --network mainnet
        env:
          ETHERSCAN_API_KEY: ${{ secrets.ETHERSCAN_API_KEY }}
```

### Smart Contract Deployment Automation

Safely automating smart contract deployment:

```javascript
// scripts/deploy-with-verification.js
const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  // Get network details
  const { chainId } = await ethers.provider.getNetwork();
  console.log(`Deploying to chain ID: ${chainId}`);
  
  // Load configuration for this network
  const networkConfig = loadNetworkConfig(chainId);
  
  // Get deployer account
  const [deployer] = await ethers.getSigners();
  console.log(`Deploying with account: ${deployer.address}`);
  
  // Log deployer balance
  const balance = await deployer.getBalance();
  console.log(`Deployer balance: ${ethers.utils.formatEther(balance)} ETH`);
  
  // Deploy contracts
  const deployedContracts = {};
  
  // Deploy token contract
  console.log("Deploying ProzToken...");
  const ProzToken = await ethers.getContractFactory("ProzToken");
  const token = await ProzToken.deploy(
    "ProzChain Token",
    "PROZ",
    networkConfig.initialSupply
  );
  await token.deployed();
  deployedContracts.token = token.address;
  console.log(`ProzToken deployed to: ${token.address}`);
  
  // Deploy governance contract
  console.log("Deploying Governance...");
  const Governance = await ethers.getContractFactory("Governance");
  const governance = await Governance.deploy(
    token.address,
    networkConfig.votingThreshold,
    networkConfig.votingPeriod
  );
  await governance.deployed();
  deployedContracts.governance = governance.address;
  console.log(`Governance deployed to: ${governance.address}`);
  
  // Save deployment info
  saveDeploymentInfo(chainId, deployedContracts);
  
  // Return deployed contracts for testing
  return deployedContracts;
}

function loadNetworkConfig(chainId) {
  const configPath = path.join(__dirname, "../config/deployment-config.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  
  const networkName = getNetworkName(chainId);
  if (!config[networkName]) {
    throw new Error(`No configuration found for network: ${networkName} (chainId: ${chainId})`);
  }
  
  return config[networkName];
}

function getNetworkName(chainId) {
  const networks = {
    1: "mainnet",
    4: "rinkeby",
    5: "goerli",
    42: "kovan",
    246: "prozchain",
    24601: "prozchain-testnet",
    31337: "hardhat",
  };
  
  return networks[chainId] || `unknown-${chainId}`;
}

function saveDeploymentInfo(chainId, contracts) {
  const networkName = getNetworkName(chainId);
  const deploymentDir = path.join(__dirname, "../deployments");
  
  // Create deployment directory if it doesn't exist
  if (!fs.existsSync(deploymentDir)) {
    fs.mkdirSync(deploymentDir, { recursive: true });
  }
  
  // Create network directory
  const networkDir = path.join(deploymentDir, networkName);
  if (!fs.existsSync(networkDir)) {
    fs.mkdirSync(networkDir, { recursive: true });
  }
  
  // Write contract addresses
  const addressesPath = path.join(networkDir, "addresses.json");
  fs.writeFileSync(
    addressesPath,
    JSON.stringify(contracts, null, 2)
  );
  
  // Write deployment metadata
  const metadataPath = path.join(networkDir, "metadata.json");
  fs.writeFileSync(
    metadataPath,
    JSON.stringify({
      timestamp: new Date().toISOString(),
      deployer: process.env.DEPLOYER_ADDRESS || "unknown",
      version: process.env.VERSION || "unversioned"
    }, null, 2)
  );
  
  console.log(`Deployment information saved to ${networkDir}`);
}

// Execute deployment
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}

module.exports = { main };
```

### Release Versioning

Maintaining versioning for blockchain deployments:

```javascript
// scripts/update-version.js
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

// Update version numbers across project files
function updateVersion(version) {
  if (!version) {
    throw new Error("Version parameter is required");
  }
  
  // Validate version format
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    throw new Error("Version must be in format x.y.z");
  }
  
  console.log(`Updating project to version ${version}`);
  
  // Update package.json
  const packagePath = path.join(__dirname, "../package.json");
  const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));
  packageJson.version = version;
  fs.writeFileSync(packagePath, JSON.stringify(packageJson, null, 2));
  
  // Update version file for contracts
  const versionFilePath = path.join(__dirname, "../contracts/Version.sol");
  const versionContent = `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @dev Provides version information for all ProzChain contracts.
 */
contract Version {
    string public constant VERSION = "${version}";
}
`;
  fs.writeFileSync(versionFilePath, versionContent);
  
  // Create git tag
  try {
    execSync(`git tag v${version}`);
    console.log(`Git tag v${version} created`);
  } catch (error) {
    console.warn(`Failed to create git tag: ${error.message}`);
  }
  
  console.log("Version update complete");
}

// Execute if run directly
if (require.main === module) {
  const version = process.argv[2];
  updateVersion(version);
}

module.exports = { updateVersion };
```

## Deployment to Different Networks

### Network-Specific Configurations

Managing configurations for different blockchain networks:

```javascript
// config/deployment-config.json
{
  "mainnet": {
    "initialSupply": "100000000000000000000000000",
    "votingThreshold": "10000000000000000000000",
    "votingPeriod": 40320,
    "deployerAddress": "0x1111111111111111111111111111111111111111",
    "deploymentWaitConfirmations": 5,
    "gasSettings": {
      "maxFeePerGas": "auto",
      "maxPriorityFeePerGas": "1000000000"
    }
  },
  "goerli": {
    "initialSupply": "100000000000000000000000000",
    "votingThreshold": "1000000000000000000000",
    "votingPeriod": 40320,
    "deployerAddress": "0x2222222222222222222222222222222222222222",
    "deploymentWaitConfirmations": 2,
    "gasSettings": {
      "maxFeePerGas": "auto",
      "maxPriorityFeePerGas": "1000000000"
    }
  },
  "prozchain-testnet": {
    "initialSupply": "100000000000000000000000000",
    "votingThreshold": "1000000000000000000000",
    "votingPeriod": 40320,
    "deployerAddress": "0x3333333333333333333333333333333333333333",
    "deploymentWaitConfirmations": 2,
    "gasSettings": {
      "gasPrice": "10000000000"
    }
  },
  "hardhat": {
    "initialSupply": "100000000000000000000000000",
    "votingThreshold": "1000000000000000000000",
    "votingPeriod": 100,
    "deployerAddress": "0x4444444444444444444444444444444444444444",
    "deploymentWaitConfirmations": 1
  }
}
```

### Multi-network Deployment Script

Ensuring consistent deployment across networks:

```javascript
// scripts/multi-network-deploy.js
const { ethers } = require("hardhat");
const { main: deploy } = require("./deploy-with-verification");

async function multiNetworkDeploy() {
  // Get target networks from command line or environment
  const targets = getTargetNetworks();
  const results = {};
  
  // Deploy to each network sequentially
  for (const network of targets) {
    console.log(`\n\n==========================================`);
    console.log(`Deploying to ${network}...`);
    console.log(`==========================================\n`);
    
    try {
      // Switch hardhat network
      await switchNetwork(network);
      
      // Execute deployment
      const deployedContracts = await deploy();
      
      // Record results
      results[network] = {
        success: true,
        contracts: deployedContracts,
        timestamp: new Date().toISOString()
      };
      
      console.log(`\n✅ Deployment to ${network} completed`);
    } catch (error) {
      console.error(`\n❌ Deployment to ${network} failed:`, error);
      
      results[network] = {
        success: false,
        error: error.message,
        timestamp: new Date().toISOString()
      };
      
      // Exit if this is production
      if (network === "mainnet" || network === "prozchain") {
        console.error("Production deployment failed. Exiting.");
        process.exit(1);
      }
    }
  }
  
  // Final report
  console.log("\n\n==========================================");
  console.log("Deployment Summary");
  console.log("==========================================\n");
  
  for (const [network, result] of Object.entries(results)) {
    console.log(`${network}: ${result.success ? "✅ Success" : "❌ Failed"}`);
    if (result.success) {
      console.log(`  - Deployed contracts:`);
      for (const [name, address] of Object.entries(result.contracts)) {
        console.log(`    - ${name}: ${address}`);
      }
    }
  }
}

function getTargetNetworks() {
  // Check for command line arguments
  if (process.argv.length > 2) {
    return process.argv.slice(2);
  }
  
  // Check for environment variable
  if (process.env.DEPLOY_NETWORKS) {
    return process.env.DEPLOY_NETWORKS.split(",");
  }
  
  // Default to local network
  return ["hardhat"];
}

async function switchNetwork(networkName) {
  // Check if network exists in hardhat config
  const availableNetworks = await hre.config.networks;
  if (!availableNetworks[networkName]) {
    throw new Error(`Network '${networkName}' not configured in hardhat.config.js`);
  }
  
  // Set network in hardhat runtime environment
  hre.network.name = networkName;
  console.log(`Switched to network: ${networkName}`);
}

// Execute if run directly
if (require.main === module) {
  multiNetworkDeploy()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("Deployment failed:", error);
      process.exit(1);
    });
}

module.exports = { multiNetworkDeploy };
```

### Staging Environment Pipeline

Testing deployments in staging before production:

```yaml
name: Staging Deployment Pipeline

on:
  push:
    branches:
      - staging

jobs:
  deploy-staging:
    runs-on: ubuntu-latest
    environment: staging
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Tests
        run: npm test
      
      - name: Deploy to Testnet
        run: |
          npx hardhat run scripts/deploy-with-verification.js --network prozchain-testnet
        env:
          PRIVATE_KEY: ${{ secrets.TESTNET_DEPLOYER_KEY }}
      
      - name: Run Post-Deployment Tests
        run: |
          # Wait for confirmations
          sleep 60
          # Run tests against deployed contracts
          NETWORK=prozchain-testnet npm run test:integration
      
      - name: Notify Team of Successful Deployment
        if: success()
        uses: slackapi/slack-github-action@v1.23.0
        with:
          payload: |
            {
              "text": "✅ Staging deployment successful! Ready for verification.",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Staging Deployment Successful*\n\nThe contracts have been deployed to the ProzChain testnet and are ready for verification."
                  }
                },
                {
                  "type": "section",
                  "fields": [
                    {
                      "type": "mrkdwn",
                      "text": "*Environment:*\nStaging (prozchain-testnet)"
                    },
                    {
                      "type": "mrkdwn",
                      "text": "*Deployed by:*\n${{ github.actor }}"
                    }
                  ]
                },
                {
                  "type": "actions",
                  "elements": [
                    {
                      "type": "button",
                      "text": {
                        "type": "plain_text",
                        "text": "View Deployment"
                      },
                      "url": "https://testnet-explorer.prozchain.network/address/0x1234567890123456789012345678901234567890"
                    }
                  ]
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_STAGING }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK
      
      - name: Notify Team of Failed Deployment
        if: failure()
        uses: slackapi/slack-github-action@v1.23.0
        with:
          payload: |
            {
              "text": "❌ Staging deployment failed!",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Staging Deployment Failed*\n\nThe deployment to ProzChain testnet encountered errors."
                  }
                },
                {
                  "type": "section",
                  "fields": [
                    {
                      "type": "mrkdwn",
                      "text": "*Environment:*\nStaging (prozchain-testnet)"
                    },
                    {
                      "type": "mrkdwn",
                      "text": "*Triggered by:*\n${{ github.actor }}"
                    }
                  ]
                },
                {
                  "type": "actions",
                  "elements": [
                    {
                      "type": "button",
                      "text": {
                        "type": "plain_text",
                        "text": "View Workflow Run"
                      },
                      "url": "https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}"
                    }
                  ]
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_STAGING }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK
```

## Post-Deployment Verification

### Automated Verification Checks

Ensuring deployments are correctly functioning:

```javascript
// scripts/verify-deployment.js
const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function verifyDeployment() {
  console.log("Starting post-deployment verification...");
  
  // Load deployment addresses
  const deployments = loadDeploymentAddresses();
  
  // Connect to contracts
  const token = await ethers.getContractAt("ProzToken", deployments.token);
  const governance = await ethers.getContractAt("Governance", deployments.governance);
  
  // Run verification checks
  const checks = [];
  
  // Check 1: Token name and symbol
  try {
    const name = await token.name();
    const symbol = await token.symbol();
    checks.push({
      name: "Token Name and Symbol",
      success: name === "ProzChain Token" && symbol === "PROZ",
      details: `Name: ${name}, Symbol: ${symbol}`
    });
  } catch (error) {
    checks.push({
      name: "Token Name and Symbol",
      success: false,
      details: `Error: ${error.message}`
    });
  }
  
  // Check 2: Token total supply
  try {
    const totalSupply = await token.totalSupply();
    const expectedSupply = ethers.utils.parseEther("100000000");
    const isCorrect = totalSupply.eq(expectedSupply);
    checks.push({
      name: "Token Total Supply",
      success: isCorrect,
      details: `Total Supply: ${ethers.utils.formatEther(totalSupply)}, Expected: 100,000,000`
    });
  } catch (error) {
    checks.push({
      name: "Token Total Supply",
      success: false,
      details: `Error: ${error.message}`
    });
  }
  
  // Check 3: Governance token address
  try {
    const governanceToken = await governance.token();
    checks.push({
      name: "Governance Token Address",
      success: governanceToken.toLowerCase() === deployments.token.toLowerCase(),
      details: `Governance token: ${governanceToken}, Expected: ${deployments.token}`
    });
  } catch (error) {
    checks.push({
      name: "Governance Token Address",
      success: false,
      details: `Error: ${error.message}`
    });
  }
  
  // Check 4: Submit test proposal to governance
  try {
    const [signer] = await ethers.getSigners();
    
    // Check if signer has enough tokens for proposal
    let hasTokens = false;
    const balance = await token.balanceOf(signer.address);
    const threshold = await governance.proposalThreshold();
    hasTokens = balance.gte(threshold);
    
    if (!hasTokens) {
      // Skip test proposal
      checks.push({
        name: "Test Proposal Creation",
        success: true,
        details: `Skipped: Insufficient tokens (${ethers.utils.formatEther(balance)}) to meet threshold (${ethers.utils.formatEther(threshold)})`
      });
    } else {
      // Approve tokens for governance
      await token.approve(governance.address, threshold);
      
      // Create test proposal
      const callData = "0x";
      const description = "Test proposal for deployment verification";
      const tx = await governance.propose(
        [deployments.token], // targets
        [0], // values
        [callData], // calldatas
        description
      );
      const receipt = await tx.wait();
      
      // Find proposal ID from event
      const event = receipt.events.find(e => e.event === "ProposalCreated");
      const proposalId = event ? event.args.proposalId : "Unknown";
      
      checks.push({
        name: "Test Proposal Creation",
        success: true,
        details: `Successfully created proposal ID: ${proposalId}`
      });
    }
  } catch (error) {
    checks.push({
      name: "Test Proposal Creation",
      success: false,
      details: `Error: ${error.message}`
    });
  }
  
  // Output results
  console.log("\n==========================================");
  console.log("Verification Results:");
  console.log("==========================================");
  
  let allPassed = true;
  for (const check of checks) {
    const status = check.success ? "✅ PASS" : "❌ FAIL";
    console.log(`${status} | ${check.name}`);
    console.log(`      ${check.details}`);
    if (!check.success) {
      allPassed = false;
    }
  }
  
  console.log("\n==========================================");
  console.log(`Overall Status: ${allPassed ? "✅ PASS" : "❌ FAIL"}`);
  console.log("==========================================");
  
  // Return status for CI usage
  return allPassed;
}

function loadDeploymentAddresses() {
  // Get network name
  const networkName = hre.network.name;
  
  // Load deployment file
  const deploymentPath = path.join(
    __dirname,
    "../deployments",
    networkName,
    "addresses.json"
  );
  
  if (!fs.existsSync(deploymentPath)) {
    throw new Error(`No deployment found for network: ${networkName}`);
  }
  
  return JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
}

// Execute if run directly
if (require.main === module) {
  verifyDeployment()
    .then((success) => {
      process.exit(success ? 0 : 1);
    })
    .catch((error) => {
      console.error("Verification failed with error:", error);
      process.exit(1);
    });
}

module.exports = { verifyDeployment };
```

### Monitoring Deployments

Setting up monitoring for new deployments:

```javascript
// scripts/setup-deployment-monitoring.js
const axios = require('axios');
const { ethers } = require('hardhat');

async function setupDeploymentMonitoring() {
  // Load deployment addresses
  const deployments = require('../deployments/latest-deployment.json');
  
  // Configure monitoring based on environment
  const network = hre.network.name;
  const isProduction = network === "mainnet" || network === "prozchain";
  
  // Setup monitoring with different thresholds based on environment
  const alertThresholds = isProduction
    ? {
        errorRate: 0.1, // 0.1% error rate in production
        latency: 2000,  // 2 seconds max latency
        gasUsage: 80,   // 80% of block gas limit
      }
    : {
        errorRate: 1.0, // 1% error rate in test environments
        latency: 5000,  // 5 seconds max latency
        gasUsage: 90,   // 90% of block gas limit
      };
  
  // For each contract, setup monitoring
  for (const [contractName, address] of Object.entries(deployments)) {
    console.log(`Setting up monitoring for ${contractName} at ${address}`);
    
    // Create alert rules in monitoring system
    await createMonitoringRules(
      network,
      contractName,
      address,
      alertThresholds
    );
    
    // Set up transaction watching
    await setupTransactionWatching(network, contractName, address);
  }
  
  console.log("Deployment monitoring setup complete");
}

async function createMonitoringRules(network, contractName, address, thresholds) {
  // In a real implementation, this would call your monitoring API
  // This is a placeholder that logs what would happen
  
  console.log(`Creating monitoring rules for ${contractName}:`);
  console.log(`- Error rate threshold: ${thresholds.errorRate}%`);
  console.log(`- Max latency: ${thresholds.latency}ms`);
  console.log(`- Gas usage alert: ${thresholds.gasUsage}%`);
  
  // Example of calling a monitoring API
  if (process.env.MONITORING_API_KEY) {
    try {
      await axios.post(
        'https://monitoring.example.com/api/rules',
        {
          network,
          contractName,
          address,
          thresholds,
          alertRecipients: ["blockchain-alerts@example.com"]
        },
        {
          headers: {
            'Authorization': `Bearer ${process.env.MONITORING_API_KEY}`
          }
        }
      );
      console.log(`Monitoring rules created successfully for ${contractName}`);
    } catch (error) {
      console.error(`Failed to create monitoring rules: ${error.message}`);
    }
  }
}

async function setupTransactionWatching(network, contractName, address) {
  // In a real implementation, this would set up webhooks or listeners
  // This is a placeholder that logs what would happen
  
  console.log(`Setting up transaction watching for ${contractName} on ${network}`);
  
  // Example of registering a webhook
  if (process.env.BLOCKCHAIN_WATCHER_API_KEY) {
    try {
      await axios.post(
        'https://watcher.example.com/api/watch',
        {
          network,
          address,
          webhookUrl: process.env.TRANSACTION_WEBHOOK_URL,
          eventTypes: ["transaction", "event"],
          filterTopics: [] // Watch all events
        },
        {
          headers: {
            'Authorization': `Bearer ${process.env.BLOCKCHAIN_WATCHER_API_KEY}`
          }
        }
      );
      console.log(`Transaction watching set up for ${contractName}`);
    } catch (error) {
      console.error(`Failed to set up transaction watching: ${error.message}`);
    }
  }
}

// Execute if run directly
if (require.main === module) {
  setupDeploymentMonitoring()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("Setup failed with error:", error);
      process.exit(1);
    });
}

module.exports = { setupDeploymentMonitoring };
```

### Emergency Response Plan

Planning for deployment emergencies:

```markdown
# Deployment Emergency Response Plan

## Emergency Scenarios

1. **Contract Vulnerability Detected Post-Deployment**
   - Severity levels:
     - Critical: Immediate funds at risk
     - High: Potential funds at risk with specific actions
     - Medium: Logic flaw affecting functionality
     - Low: Minor issue with minimal impact

2. **Failed Deployment**
   - Incomplete deployment
   - Failed contract verification
   - Incorrect constructor parameters

3. **Network Issues**
   - Chain reorganization affecting deployment
   - Network congestion preventing confirmations
   - RPC endpoint failures

## Emergency Response Team

| Role | Responsibility | Primary Contact | Backup Contact |
|------|---------------|-----------------|----------------|
| Lead Engineer | Technical assessment & fixes | @lead-dev | @backup-dev |
| Security Analyst | Vulnerability assessment | @security-lead | @security-backup |
| Communications Lead | Stakeholder communications | @comms-lead | @comms-backup |
| Executive Decision Maker | Final decisions on major actions | @exec-lead | @exec-backup |

## Emergency Response Process

### 1. Detection and Assessment

- **Monitoring Alerts**: Automated alerts from deployment monitoring
- **Initial Assessment**: Technical team evaluates severity within 15 minutes
- **Team Notification**: Alert appropriate team members based on severity
  - Critical/High: Full emergency team
  - Medium/Low: Technical team only

### 2. Immediate Actions

#### For Critical Vulnerabilities:
1. Initiate emergency pause if available
2. Prepare emergency communications
3. Convene full response team
4. Begin developing mitigation strategy

#### For Failed Deployments:
1. Prevent further interactions with affected contracts
2. Determine cause of failure
3. Prepare retry strategy or rollback plan

### 3. Communication Plan

#### Internal Communications:
- Use dedicated emergency channel in Slack
- Emergency call for Critical/High issues
- Document all decisions and actions

#### External Communications:
- Initial notification within 1 hour for Critical issues
- Regular updates every 2 hours until resolved
- Final report after resolution

### 4. Recovery Procedures

#### For Contract Vulnerabilities:
1. Deploy emergency fix if possible
2. Implement upgrade if contract is upgradeable
3. Deploy replacement contract if necessary
4. Migrate assets if required

#### For Failed Deployments:
1. Fix deployment scripts/parameters
2. Re-deploy with corrected configuration
3. Verify successful deployment with enhanced checks

### 5. Post-Incident Actions

1. Conduct full incident review
2. Document lessons learned
3. Update deployment procedures
4. Enhance monitoring if needed
5. Conduct additional security audits if required

## Emergency Contacts

| Service | Contact Information |
|---------|---------------------|
| Security Hotline | +1-555-123-4567 |
| On-Call Engineer | on-call@prozchain.example |
| Bug Bounty Program | bounty@prozchain.example |

## Recovery Tools & Resources

- Emergency response repository: `github.com/prozchain/emergency-response`
- Contract upgrade tools: `github.com/prozchain/contract-upgrader`
- State migration scripts: `github.com/prozchain/state-migrator`
```

## Multi-Signature Deployment Process

### Safe Contract Deployment

Using multi-signature wallets for enhanced security:

```javascript
// scripts/deploy-with-multisig.js
const { ethers } = require("hardhat");
const Safe = require('@gnosis.pm/safe-core-sdk').default;
const EthersAdapter = require('@gnosis.pm/safe-ethers-lib').default;
const SafeServiceClient = require('@gnosis.pm/safe-service-client').default;

async function deployWithMultisig() {
  console.log("Starting multi-signature deployment process...");
  
  // Get the contract factory
  const ProzToken = await ethers.getContractFactory("ProzToken");
  const tokenParams = [
    "ProzChain Token",
    "PROZ",
    ethers.utils.parseEther("100000000") // 100 million tokens
  ];
  
  // Get deployment transaction data (without sending it)
  const deployTx = ProzToken.getDeployTransaction(...tokenParams);
  
  // Get the deployer account
  const [deployer] = await ethers.getSigners();
  
  // Connect to multi-sig wallet
  const ethAdapter = new EthersAdapter({
    ethers,
    signer: deployer
  });
  
  // Load Safe multi-sig
  const safeAddress = process.env.MULTISIG_ADDRESS;
  if (!safeAddress) {
    throw new Error("MULTISIG_ADDRESS environment variable not set");
  }
  
  const safe = await Safe.create({
    ethAdapter,
    safeAddress
  });
  
  // Load Safe transaction service
  const safeService = new SafeServiceClient({
    txServiceUrl: getSafeServiceUrl(),
    ethAdapter
  });
  
  console.log(`Connected to multi-sig wallet: ${safeAddress}`);
  console.log(`Creating deployment transaction for ProzToken...`);
  
  // Create Safe transaction
  const safeTransaction = await safe.createTransaction({
    to: ethers.constants.AddressZero, // Contract creation
    data: deployTx.data,
    value: "0"
  });
  
  // Sign transaction with first signer
  const safeTxHash = await safe.getTransactionHash(safeTransaction);
  const senderSignature = await safe.signTransaction(safeTransaction);
  
  // Propose the transaction to the Safe service
  await safeService.proposeTransaction({
    safeAddress,
    safeTransaction,
    safeTxHash,
    senderAddress: deployer.address,
    senderSignature: senderSignature.data
  });
  
  console.log(`Transaction proposed to multi-sig wallet`);
  console.log(`Safe transaction hash: ${safeTxHash}`);
  console.log(`\nNext steps:`);
  console.log(`1. Other signers must review and sign the transaction`);
  console.log(`2. After reaching the threshold, any signer can execute the transaction`);
  console.log(`3. Run verification script after execution`);
  
  // Return the transaction hash for tracking
  return safeTxHash;
}

function getSafeServiceUrl() {
  // Get the appropriate Safe service URL based on the network
  const network = hre.network.name;
  
  const serviceUrls = {
    mainnet: "https://safe-transaction-mainnet.safe.global",
    goerli: "https://safe-transaction-goerli.safe.global",
    sepolia: "https://safe-transaction-sepolia.safe.global",
    polygon: "https://safe-transaction-polygon.safe.global",
    "prozchain-testnet": "https://transaction-service.prozchain.network"
  };
  
  return serviceUrls[network] || serviceUrls.mainnet;
}

// Execute if run directly
if (require.main === module) {
  deployWithMultisig()
    .then((txHash) => {
      console.log(`Multi-sig deployment proposed with hash: ${txHash}`);
      process.exit(0);
    })
    .catch((error) => {
      console.error("Deployment failed:", error);
      process.exit(1);
    });
}

module.exports = { deployWithMultisig };
```

### Multi-Signature Approval Workflow

Integrating multi-signature approvals into CI/CD:

```yaml
name: Multi-Signature Deployment

on:
  workflow_dispatch:
    inputs:
      network:
        description: 'Target network for deployment'
        required: true
        default: 'goerli'
        type: choice
        options:
          - goerli
          - sepolia
          - prozchain-testnet
          - mainnet
      
      description:
        description: 'Deployment description'
        required: true
        type: string

jobs:
  prepare-deployment:
    runs-on: ubuntu-latest
    environment: ${{ inputs.network }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Tests
        run: npm test
      
      - name: Create Multi-Sig Transaction
        id: create_tx
        run: |
          TX_HASH=$(node scripts/deploy-with-multisig.js)
          echo "tx_hash=$TX_HASH" >> $GITHUB_OUTPUT
        env:
          MULTISIG_ADDRESS: ${{ secrets.MULTISIG_ADDRESS }}
          PRIVATE_KEY: ${{ secrets.DEPLOYER_PRIVATE_KEY }}
      
      - name: Create GitHub Issue for Signatures
        uses: actions/github-script@v6
        with:
          github-token: ${{ secrets.PAT_TOKEN }}
          script: |
            const txHash = '${{ steps.create_tx.outputs.tx_hash }}';
            const network = '${{ inputs.network }}';
            const description = '${{ inputs.description }}';
            
            const safeAppUrl = network === 'mainnet'
              ? `https://app.safe.global/transactions/queue?safe=eth:${process.env.MULTISIG_ADDRESS}`
              : `https://app.safe.global/transactions/queue?safe=${network}:${process.env.MULTISIG_ADDRESS}`;
            
            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `[SIGNATURE REQUIRED] Deployment to ${network}`,
              body: `## Multi-Signature Deployment Request
              
              A new deployment has been proposed to the ${network} network and requires signatures.
              
              **Deployment Description**: ${description}
              
              **Transaction Hash**: \`${txHash}\`
              
              ### Required Actions
              
              1. Review the deployment transaction in Safe
              2. Sign the transaction if approved
              
              ### Links
              
              - [View in Safe App](${safeAppUrl})
              - [View in Explorer](https://${network === 'mainnet' ? '' : network + '.'}etherscan.io/tx/${txHash})
              
              ### Signers Status
              
              - [ ] Signer 1
              - [ ] Signer 2
              - [ ] Signer 3
              
              Once enough signatures are collected, the transaction can be executed from the Safe interface.
              
              ### Post-Execution
              
              After execution, run the verification script:
              \`\`\`
              NETWORK=${network} npm run verify-deployment
              \`\`\`
              `,
              labels: ['multi-sig', 'deployment', network]
            });
            
            console.log(`Created issue #${issue.data.number} for signature collection`);
        env:
          MULTISIG_ADDRESS: ${{ secrets.MULTISIG_ADDRESS }}
      
      - name: Notify Team
        uses: slackapi/slack-github-action@v1.23.0
        with:
          payload: |
            {
              "text": "🔐 Multi-signature deployment requires approval",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Multi-Signature Deployment Requires Approval*\n\nA new deployment to ${network} has been proposed and requires signatures from authorized signers."
                  }
                },
                {
                  "type": "section",
                  "fields": [
                    {
                      "type": "mrkdwn",
                      "text": "*Network:*\n${{ inputs.network }}"
                    },
                    {
                      "type": "mrkdwn",
                      "text": "*Proposed by:*\n${{ github.actor }}"
                    }
                  ]
                },
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Description:*\n${{ inputs.description }}"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK
```

## Conclusion

Continuous deployment for blockchain applications presents unique challenges due to the immutable nature of deployed smart contracts and the financial implications of deployment errors. By implementing a structured deployment pipeline with proper testing, verification, and multi-signature approvals, teams can safely and efficiently deploy updates to blockchain networks.

The strategies outlined in this chapter—from automated deployment scripts to post-deployment verification—provide a robust framework for managing deployments across different environments. This approach helps ensure that only thoroughly tested, secure code reaches production networks, reducing the risk of costly errors.

In the next chapter, we'll explore best practices for maintaining and evolving the testing framework as the ProzChain project grows and changes.
