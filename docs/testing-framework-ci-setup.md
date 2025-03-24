# CI Pipeline Setup

## Overview

Setting up an effective CI pipeline is the foundation for automated testing of blockchain applications. This chapter covers the initial configuration of CI systems, from basic setups to advanced multi-stage pipelines tailored for ProzChain development.

## Basic CI Configuration

### Choosing a CI Platform

Several CI platforms are suitable for blockchain development:

1. **GitHub Actions**: Tightly integrated with GitHub repositories
   - Native integration with repository events
   - Marketplace with blockchain-specific actions
   - Secure secrets management

2. **CircleCI**: Flexible configuration with Docker support
   - Custom resource allocation
   - Orb ecosystem for reusable configurations
   - Advanced caching mechanisms

3. **Jenkins**: Self-hosted with extensive customization
   - Complete control over execution environment
   - Extensive plugin ecosystem
   - Support for complex pipelines

4. **GitLab CI**: Integrated with GitLab repositories
   - Auto DevOps capabilities
   - Built-in container registry
   - Kubernetes integration

### Example GitHub Actions Workflow

```yaml
name: ProzChain Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Linter
        run: npm run lint
      
      - name: Compile Contracts
        run: npm run compile
      
      - name: Run Tests
        run: npm test
      
      - name: Generate Coverage Reports
        run: npm run coverage
      
      - name: Archive Code Coverage Results
        uses: actions/upload-artifact@v3
        with:
          name: code-coverage-report
          path: coverage/
```

### Setting Up Repository Secrets

Secure management of sensitive information:

1. **Navigate to repository settings**
   - Go to your GitHub repository
   - Select "Settings" > "Secrets and variables" > "Actions"

2. **Add necessary secrets**:
   - RPC endpoints (e.g., `PROZCHAIN_TESTNET_RPC`)
   - Private keys for test accounts (e.g., `TEST_PRIVATE_KEY`)
   - API keys for services (e.g., `ETHERSCAN_API_KEY`)

3. **Best practices for secret management**:
   - Use dedicated test accounts with limited funds
   - Rotate keys regularly
   - Use environment-specific secrets

## Advanced Pipeline Configuration

### Multi-Stage Pipeline

Creating sophisticated CI workflows with dependencies:

```yaml
name: ProzChain Advanced Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  lint:
    name: Code Quality Checks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      - run: npm ci
      - name: Run Linter
        run: npm run lint
      - name: Run Solhint
        run: npx solhint 'contracts/**/*.sol'
  
  compile:
    name: Compile Contracts
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      - run: npm ci
      - name: Compile Contracts
        run: npm run compile
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: compiled-contracts
          path: artifacts/
  
  unit-tests:
    name: Unit Tests
    needs: compile
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      - run: npm ci
      - name: Download Compiled Contracts
        uses: actions/download-artifact@v3
        with:
          name: compiled-contracts
          path: artifacts/
      - name: Run Unit Tests
        run: npm run test:unit
  
  integration-tests:
    name: Integration Tests
    needs: compile
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      - run: npm ci
      - name: Download Compiled Contracts
        uses: actions/download-artifact@v3
        with:
          name: compiled-contracts
          path: artifacts/
      - name: Run Integration Tests
        run: npm run test:integration
  
  coverage:
    name: Test Coverage
    needs: [unit-tests, integration-tests]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      - run: npm ci
      - name: Generate Coverage Report
        run: npm run coverage
      - name: Upload Coverage Reports
        uses: actions/upload-artifact@v3
        with:
          name: coverage-reports
          path: coverage/
  
  security:
    name: Security Analysis
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
        run: slither . --json slither-report.json
      - name: Upload Security Report
        uses: actions/upload-artifact@v3
        with:
          name: security-reports
          path: slither-report.json
```

### Testing Matrix Strategy

Testing across multiple configurations:

```yaml
jobs:
  test:
    name: Test on Node ${{ matrix.node-version }} and ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        node-version: [14.x, 16.x, 18.x]
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
      - uses: actions/checkout@v3
      - name: Use Node.js ${{ matrix.node-version }}
        uses: actions/setup-node@v3
        with:
          node-version: ${{ matrix.node-version }}
      - name: Install Dependencies
        run: npm ci
      - name: Run Tests
        run: npm test
```

### Caching and Performance Optimization

Improving CI pipeline efficiency:

```yaml
steps:
  - uses: actions/checkout@v3
  
  - name: Setup Node.js
    uses: actions/setup-node@v3
    with:
      node-version: '16'
      cache: 'npm'
  
  - name: Cache Hardhat Network Fork
    uses: actions/cache@v3
    with:
      path: cache/hardhat-network-fork
      key: hardhat-network-fork-${{ runner.os }}-${{ hashFiles('hardhat.config.js') }}
      restore-keys: |
        hardhat-network-fork-${{ runner.os }}-
  
  - name: Cache Solidity Compiler
    uses: actions/cache@v3
    with:
      path: ~/.cache/hardhat-nodejs/compilers
      key: solidity-compilers-${{ runner.os }}-${{ hashFiles('hardhat.config.js') }}
  
  - name: Install Dependencies
    run: npm ci
  
  - name: Run Tests
    run: npm test
```

## CI for Different Environments

### Testing Against Multiple Networks

Configuring CI to test across different blockchain networks:

```yaml
jobs:
  network-tests:
    name: Test on ${{ matrix.network }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        network: [hardhat, prozchain-testnet, ethereum-goerli]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Set Network Configuration
        run: |
          if [ "${{ matrix.network }}" = "hardhat" ]; then
            echo "Using local Hardhat network"
          elif [ "${{ matrix.network }}" = "prozchain-testnet" ]; then
            echo "PROZCHAIN_RPC_URL=${{ secrets.PROZCHAIN_TESTNET_RPC }}" >> $GITHUB_ENV
            echo "PROZCHAIN_PRIVATE_KEY=${{ secrets.TESTNET_PRIVATE_KEY }}" >> $GITHUB_ENV
          elif [ "${{ matrix.network }}" = "ethereum-goerli" ]; then
            echo "ETH_RPC_URL=${{ secrets.GOERLI_RPC_URL }}" >> $GITHUB_ENV
            echo "ETH_PRIVATE_KEY=${{ secrets.TESTNET_PRIVATE_KEY }}" >> $GITHUB_ENV
          fi
      
      - name: Run Tests on ${{ matrix.network }}
        run: npm run test:${{ matrix.network }}
```

### Docker-Based Environment

Using containers for consistent test environments:

```yaml
jobs:
  docker-tests:
    name: Docker Container Tests
    runs-on: ubuntu-latest
    
    services:
      ganache:
        image: trufflesuite/ganache:latest
        ports:
          - 8545:8545
        options: >-
          --chain.chainId 1337
          --chain.networkId 1337
          --wallet.deterministic
      
      ipfs:
        image: ipfs/kubo:latest
        ports:
          - 5001:5001
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Tests with Docker Services
        run: npm run test:docker
        env:
          ETHEREUM_RPC_URL: http://localhost:8545
          IPFS_API_URL: http://localhost:5001
```

### Full-Stack Testing

Setting up CI for end-to-end application testing:

```yaml
jobs:
  fullstack-tests:
    name: Full-Stack E2E Tests
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: |
          npm ci
          cd frontend
          npm ci
          cd ../backend
          npm ci
      
      - name: Start Local Blockchain
        run: npx hardhat node &
      
      - name: Deploy Contracts
        run: npx hardhat run scripts/deploy.js --network localhost
      
      - name: Start Backend Server
        run: |
          cd backend
          npm run start:test &
          sleep 5
      
      - name: Start Frontend
        run: |
          cd frontend
          npm start &
          sleep 10
      
      - name: Run Cypress Tests
        uses: cypress-io/github-action@v4
        with:
          working-directory: frontend
          browser: chrome
```

## Conclusion

A well-configured CI pipeline is essential for maintaining code quality and preventing regressions in blockchain applications. By adopting the configurations outlined in this chapter, teams can automate testing across different environments, ensuring that code changes are thoroughly validated before deployment.
