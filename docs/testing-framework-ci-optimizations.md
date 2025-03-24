# CI Pipeline Optimizations

## Overview

CI pipelines for blockchain projects can be time-consuming and resource-intensive. This chapter explores strategies to optimize CI pipelines, improve performance, and ensure reliable, deterministic test results for ProzChain applications.

## Performance Improvements

### Resource Usage Optimization

Strategies for efficient resource utilization:

```yaml
jobs:
  performance-optimized-tests:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      # Use setup-node caching
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
          cache: 'npm'
      
      # Cache RPC requests in tests
      - name: Cache RPC Data
        uses: actions/cache@v3
        with:
          path: .rpc-cache
          key: ${{ runner.os }}-rpc-${{ hashFiles('test/**/*.js') }}
      
      # Install only production dependencies for tests
      - name: Install Dependencies
        run: npm ci --production
      
      # Run tests with resource constraints
      - name: Run Tests with Resource Limits
        run: node --max-old-space-size=4096 node_modules/.bin/hardhat test
        env:
          HARDHAT_MEMORY_LIMIT: "4096"
          USE_RPC_CACHE: "true"
          REPORT_GAS: "true"
```

Example RPC caching implementation:

```javascript
// test/helpers/rpc-cache.js
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Setup cache directory
const CACHE_DIR = path.join(process.cwd(), '.rpc-cache');
if (!fs.existsSync(CACHE_DIR)) {
  fs.mkdirSync(CACHE_DIR, { recursive: true });
}

// Cache middleware for ethers provider
function createCachingProvider(provider) {
  // Only use cache if enabled
  if (process.env.USE_RPC_CACHE !== 'true') {
    return provider;
  }

  const originalSend = provider.send.bind(provider);
  
  provider.send = async function(method, params) {
    // Skip caching for state-changing methods
    const readOnlyMethods = [
      'eth_call', 'eth_getBalance', 'eth_getCode', 'eth_getTransactionCount',
      'eth_getStorageAt', 'eth_getBlockByNumber', 'eth_getBlockByHash',
      'eth_getTransactionReceipt', 'eth_getTransactionByHash'
    ];
    
    if (!readOnlyMethods.includes(method)) {
      return originalSend(method, params);
    }
    
    // Create cache key from method and params
    const cacheKey = crypto
      .createHash('md5')
      .update(`${method}-${JSON.stringify(params)}`)
      .digest('hex');
    
    const cacheFile = path.join(CACHE_DIR, cacheKey);
    
    // Check cache
    if (fs.existsSync(cacheFile)) {
      try {
        const cached = JSON.parse(fs.readFileSync(cacheFile, 'utf8'));
        return cached;
      } catch (err) {
        console.warn(`Cache read error for ${method}:`, err.message);
      }
    }
    
    // Call RPC and cache result
    const result = await originalSend(method, params);
    
    try {
      fs.writeFileSync(cacheFile, JSON.stringify(result));
    } catch (err) {
      console.warn(`Cache write error for ${method}:`, err.message);
    }
    
    return result;
  };
  
  return provider;
}

module.exports = { createCachingProvider };
```

### Test Prioritization

Running important tests first:

```javascript
// scripts/prioritize-tests.js
const fs = require('fs');
const path = require('path');

// Configuration
const HIGH_PRIORITY_PATTERNS = [
  'security', 'critical', 'core'
];

const MEDIUM_PRIORITY_PATTERNS = [
  'integration', 'workflow'
];

// Find all test files
const testDir = path.join(__dirname, '../test');
const testFiles = findFiles(testDir, '.test.js');

// Group by priority
const highPriority = testFiles.filter(file => 
  HIGH_PRIORITY_PATTERNS.some(pattern => file.includes(pattern))
);

const mediumPriority = testFiles.filter(file => 
  !highPriority.includes(file) && 
  MEDIUM_PRIORITY_PATTERNS.some(pattern => file.includes(pattern))
);

const lowPriority = testFiles.filter(file => 
  !highPriority.includes(file) && !mediumPriority.includes(file)
);

// Create prioritized test list
const prioritizedTests = [
  ...highPriority,
  ...mediumPriority,
  ...lowPriority
];

// Output the list
console.log(prioritizedTests.join('\n'));

// Helper function to find files recursively
function findFiles(dir, extension) {
  let results = [];
  const files = fs.readdirSync(dir);
  
  for (const file of files) {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      results = results.concat(findFiles(filePath, extension));
    } else if (file.endsWith(extension)) {
      results.push(filePath);
    }
  }
  
  return results;
}
```

Example Mocha configuration using prioritized tests:

```javascript
// .mocharc.js
const { execSync } = require('child_process');

// Get prioritized test files
const testFiles = execSync('node scripts/prioritize-tests.js')
  .toString()
  .trim()
  .split('\n');

module.exports = {
  spec: testFiles,
  timeout: 30000,
  // other config...
};
```

### Parallel Execution

Maximizing execution efficiency:

```yaml
jobs:
  parallel-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        partition: [1, 2, 3, 4, 5, 6]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Tests (Partition ${{ matrix.partition }})
        run: npm test -- --partition=${{ matrix.partition }} --partition-total=6
```

Example implementation for test partitioning:

```javascript
// scripts/test-partitioning.js
const Mocha = require('mocha');
const glob = require('glob');
const yargs = require('yargs/yargs');
const { hideBin } = require('yargs/helpers');

// Parse command line arguments
const argv = yargs(hideBin(process.argv))
  .option('partition', {
    type: 'number',
    description: 'Which partition to run (1-based)',
    default: 1
  })
  .option('partition-total', {
    type: 'number',
    description: 'Total number of partitions',
    default: 1
  })
  .argv;

// Validate partition arguments
const partition = argv.partition;
const totalPartitions = argv['partition-total'];

if (partition < 1 || partition > totalPartitions) {
  console.error(`Invalid partition ${partition}/${totalPartitions}`);
  process.exit(1);
}

// Get all test files
const testFiles = glob.sync('test/**/*.test.js');

// Distribute files across partitions
const partitionSize = Math.ceil(testFiles.length / totalPartitions);
const startIndex = (partition - 1) * partitionSize;
const endIndex = Math.min(startIndex + partitionSize, testFiles.length);

const partitionFiles = testFiles.slice(startIndex, endIndex);

console.log(`Running partition ${partition}/${totalPartitions} with ${partitionFiles.length} test files`);
console.log(`Files: ${partitionFiles.join(', ')}`);

// Run the tests in this partition
const mocha = new Mocha();
partitionFiles.forEach(file => mocha.addFile(file));

mocha.run(failures => {
  process.exitCode = failures ? 1 : 0;
});
```

## Reproducibility and Determinism

### Deterministic Test Environment

Ensuring consistent test execution:

```yaml
jobs:
  deterministic-tests:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      # Use fixed timestamp for tests
      - name: Set Fixed Test Date
        run: |
          echo "TEST_TIMESTAMP=1640995200000" >> $GITHUB_ENV
      
      # Use deterministic blockchain settings
      - name: Start Deterministic Blockchain
        run: |
          npx hardhat node \
            --hostname 127.0.0.1 \
            --port 8545 \
            --fork https://eth-mainnet.alchemyapi.io/v2/${{ secrets.ALCHEMY_KEY }}@15000000 \
            --fork-block-number 15000000 \
            --mnemonic "test test test test test test test test test test test junk" \
            --network-id 1337 \
            --no-rate-limit &
          sleep 5
      
      - name: Run Tests with Fixed Parameters
        run: npm test
        env:
          RANDOMIZE_TESTS: "false"
          HARDHAT_NETWORK: "localhost"
          MOCHA_SEED: "12345"
          TEST_REPETITIONS: "3"
```

Example helper for deterministic testing:

```javascript
// test/helpers/deterministic.js
const { ethers } = require('hardhat');

/**
 * Setup a deterministic test environment
 */
async function setupDeterministicTests() {
  // Use fixed timestamp if provided
  if (process.env.TEST_TIMESTAMP) {
    const timestamp = parseInt(process.env.TEST_TIMESTAMP);
    await ethers.provider.send("evm_setNextBlockTimestamp", [timestamp]);
  }
  
  // Set fixed gas price for all transactions
  const gasPrice = ethers.utils.parseUnits('50', 'gwei');
  const signer = await ethers.getSigners();
  signer.forEach(s => {
    s.sendTransaction = async (tx) => {
      return ethers.Signer.prototype.sendTransaction.call(
        s, 
        { ...tx, gasPrice }
      );
    };
  });
  
  return {
    // Helper to advance time in a deterministic way
    advanceTimeByDays: async (days) => {
      const seconds = days * 86400;
      await ethers.provider.send("evm_increaseTime", [seconds]);
      await ethers.provider.send("evm_mine");
    },
    
    // Helper to generate deterministic random values
    deterministicRandom: (seed) => {
      // Simple deterministic random function
      const finalSeed = seed || process.env.MOCHA_SEED || '12345';
      let s = parseInt(finalSeed);
      return () => {
        s = (s * 9301 + 49297) % 233280;
        return s / 233280;
      };
    }
  };
}

module.exports = { setupDeterministicTests };
```

### Snapshot-Based Testing

Using snapshots for state management:

```javascript
// test/helpers/snapshot.js
const hre = require("hardhat");

let snapshotId;

async function takeSnapshot() {
  const snapshot = await hre.network.provider.send("evm_snapshot");
  return snapshot;
}

async function revertToSnapshot(id) {
  await hre.network.provider.send("evm_revert", [id]);
}

// Global setup for test suite
beforeEach(async function() {
  // Take fresh snapshot before each test
  snapshotId = await takeSnapshot();
});

afterEach(async function() {
  // Revert to clean state after each test
  await revertToSnapshot(snapshotId);
});

module.exports = {
  takeSnapshot,
  revertToSnapshot
};
```

### Controlling External Dependencies

Managing external dependencies for consistent testing:

```javascript
// test/setup/mockServices.js
const nock = require('nock');
const fs = require('fs');
const path = require('path');

// Load mock responses from fixture files
const priceFeedResponse = JSON.parse(
  fs.readFileSync(path.join(__dirname, 'fixtures', 'coinmarketcap_response.json'), 'utf8')
);

// Mock external API dependencies
function setupMocks() {
  // Disable real HTTP requests
  nock.disableNetConnect();
  nock.enableNetConnect('127.0.0.1');
  
  // Mock price API
  nock('https://api.coinmarketcap.com')
    .persist()
    .get('/v2/cryptocurrency/quotes/latest')
    .query(true)
    .reply(200, priceFeedResponse);
  
  // Mock IPFS API
  nock('https://ipfs.infura.io:5001')
    .persist()
    .post('/api/v0/add')
    .reply(200, { Hash: 'QmUNLLsPACCz1vLxQVkXqqLX5R1X345qqfHbsf67hvA3Nn' });
  
  // Mock blockchain explorer API
  nock('https://api-testnet.etherscan.io')
    .persist()
    .get('/api')
    .query(true)
    .reply(function(uri, requestBody) {
      const params = new URLSearchParams(uri.split('?')[1]);
      if (params.get('module') === 'contract' && params.get('action') === 'getabi') {
        return [200, { status: '1', message: 'OK', result: '[]' }];
      }
      return [404];
    });
}

// Restore real HTTP requests
function tearDownMocks() {
  nock.cleanAll();
  nock.enableNetConnect();
}

module.exports = {
  setupMocks,
  tearDownMocks
};
```

## Build Optimization

### Smart Compilation

Avoiding unnecessary contract compilation:

```javascript
// scripts/smart-compile.js
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const crypto = require('crypto');

// Get all Solidity files
const contractsDir = path.join(__dirname, '../contracts');
const solidityFiles = findFiles(contractsDir, '.sol');

// Calculate hash of all contract files
const contentHashes = solidityFiles.map(file => {
  const content = fs.readFileSync(file, 'utf8');
  return crypto.createHash('md5').update(content).digest('hex');
});

// Also include compiler settings in the hash
const hardhatConfig = require('../hardhat.config');
const compilerSettings = JSON.stringify({
  version: hardhatConfig.solidity.version,
  settings: hardhatConfig.solidity.settings
});
contentHashes.push(crypto.createHash('md5').update(compilerSettings).digest('hex'));

// Create combined hash
const combinedHash = crypto
  .createHash('md5')
  .update(contentHashes.join(''))
  .digest('hex');

// Check against previous hash
const hashFile = path.join(__dirname, '../.compile-hash');
let shouldCompile = true;

if (fs.existsSync(hashFile)) {
  const previousHash = fs.readFileSync(hashFile, 'utf8');
  shouldCompile = previousHash !== combinedHash;
}

// Compile if needed
if (shouldCompile) {
  console.log('Changes detected in contracts, compiling...');
  execSync('npx hardhat compile', { stdio: 'inherit' });
  fs.writeFileSync(hashFile, combinedHash);
} else {
  console.log('No changes in contracts, skipping compilation');
}

// Helper function to find files recursively
function findFiles(dir, extension) {
  let results = [];
  const files = fs.readdirSync(dir);
  
  for (const file of files) {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      results = results.concat(findFiles(filePath, extension));
    } else if (file.endsWith(extension)) {
      results.push(filePath);
    }
  }
  
  return results;
}
```

### Dependency Optimization

Managing dependencies efficiently:

```yaml
steps:
  - name: Setup Node.js
    uses: actions/setup-node@v3
    with:
      node-version: '16'
  
  # Use lockfile for deterministic installs
  - name: Get npm cache directory
    id: npm-cache-dir
    run: |
      echo "dir=$(npm config get cache)" >> $GITHUB_OUTPUT
  
  - name: Cache npm dependencies
    uses: actions/cache@v3
    with:
      path: ${{ steps.npm-cache-dir.outputs.dir }}
      key: ${{ runner.os }}-npm-${{ hashFiles('**/package-lock.json') }}
      restore-keys: |
        ${{ runner.os }}-npm-
  
  # Install dev dependencies only for compilation
  - name: Install compile dependencies
    if: steps.get-changed-files.outputs.types == 'contracts'
    run: npm ci --only=dev
  
  # Install full dependencies for testing
  - name: Install all dependencies for tests
    if: steps.get-changed-files.outputs.types != 'contracts'
    run: npm ci
```

## Conclusion

Optimizing CI pipelines is essential for efficient blockchain development. By implementing the strategies in this chapter, teams can achieve faster build times, more reliable test results, and a more effective development process overall.

The techniques covered—resource optimization, test prioritization, reproducibility, and build improvements—work together to create a CI pipeline that supports rapid development while maintaining high quality standards.

Next, we'll explore how to configure CI for different blockchain networks, addressing the unique challenges of testing across multiple chains and environments.
