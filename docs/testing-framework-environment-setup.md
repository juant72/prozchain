# Testing Environment Setup

## Development Environment Configuration

Setting up a proper development environment is the foundation for effective testing. This section covers the tools and configurations required for ProzChain development and testing.

### Required Software

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | >=16.0.0 | JavaScript runtime |
| npm | >=8.0.0 | Package management |
| Python | 3.8+ | Required for some dependencies |
| Solidity | 0.8.17 | Smart contract development |
| Git | 2.0+ | Version control |
| Docker | 20.0+ | Container management |
| Ganache | 7.0+ | Local blockchain environment |

### Initial Setup

1. **Install the required software** listed above

2. **Clone the repository**:
   ```bash
   git clone https://github.com/prozchain/prozchain-core.git
   cd prozchain-core
   ```

3. **Install dependencies**:
   ```bash
   npm install
   ```

4. **Setup environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

5. **Verify installation**:
   ```bash
   npm run test:basic
   ```

### IDE Configuration

#### VS Code Setup

1. **Install recommended extensions**:
   - Solidity (by Juan Blanco)
   - ESLint
   - Prettier
   - EditorConfig
   - Hardhat for VS Code

2. **Configure settings**:
   ```json
   {
     "editor.formatOnSave": true,
     "solidity.compileUsingRemoteVersion": "v0.8.17",
     "solidity.linter": "solhint",
     "javascript.format.enable": false,
     "typescript.format.enable": false,
     "editor.codeActionsOnSave": {
       "source.fixAll.eslint": true
     }
   }
   ```

#### Other IDEs

- **Jetbrains IDE (WebStorm/IntelliJ)**:
  - Install the Solidity plugin
  - Configure ESLint and Prettier integrations
  - Import the `.editorconfig` settings

## Testing Dependencies and Prerequisites

### Core Testing Libraries

ProzChain uses these primary testing libraries:

- **Hardhat**: Ethereum development environment
- **Chai**: Assertion library
- **Mocha**: Test runner
- **Ethers.js**: Ethereum interaction library
- **Sinon**: Spies, stubs, and mocks
- **Hardhat Network Helpers**: Testing utilities

### Installing Test Dependencies

Most dependencies are included in the main `package.json`, but you can install testing-specific dependencies with:

```bash
npm install --save-dev hardhat @nomiclabs/hardhat-ethers @nomiclabs/hardhat-waffle ethereum-waffle chai
```

### Test Networks Configuration

Configure multiple networks in your `hardhat.config.js`:

```javascript
require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-ethers");

// Load environment variables
require("dotenv").config();

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
    // Local development network
    hardhat: {
      chainId: 31337,
      mining: {
        auto: true,
        interval: 0
      }
    },
    // Local ganache instance
    ganache: {
      url: "http://127.0.0.1:8545",
      accounts: {
        mnemonic: process.env.GANACHE_MNEMONIC || "test test test test test test test test test test test junk"
      }
    },
    // ProzChain test network
    prozchain_testnet: {
      url: process.env.PROZCHAIN_TESTNET_RPC || "https://testnet-rpc.prozchain.com",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    }
  },
  // Testing configuration
  mocha: {
    timeout: 60000 // 1 minute timeout for tests
  }
};
```

## Local Testing Environment Setup

### Running a Local Development Blockchain

Start a local development blockchain for testing:

```bash
# Using Hardhat's built-in network
npx hardhat node

# Using Ganache CLI
npx ganache --deterministic
```

### Database Setup for Integration Tests

1. **Using Docker for database services**:
   ```bash
   docker-compose up -d db
   ```

2. **Run database migrations**:
   ```bash
   npm run db:migrate
   ```

3. **Seed test data**:
   ```bash
   npm run db:seed:test
   ```

### API Service Setup

For integration tests involving API services:

```bash
# Start API services with test configuration
npm run api:start:test
```

### Environment Configuration for Different Test Types

Create test-specific environment files:

- `.env.test`: Standard test environment
- `.env.integration`: Integration test environment
- `.env.e2e`: End-to-end test environment

## Docker-Based Testing Environments

ProzChain provides Docker configurations for consistent testing environments across different developer machines and CI systems.

### Starting the Test Environment

```bash
# Start all required services
docker-compose -f docker-compose.test.yml up -d

# Run tests in the container
docker-compose -f docker-compose.test.yml run test
```

### Available Docker Test Environments

- **Basic test environment**: `docker-compose.test.yml`
- **Full network simulation**: `docker-compose.network.yml`
- **Security testing**: `docker-compose.security.yml`
- **Performance testing**: `docker-compose.performance.yml`

### Custom Environment Configuration

You can customize environment variables for test containers:

```bash
# Create a custom environment file
cp .env.docker.example .env.docker.custom

# Edit the values in .env.docker.custom

# Run with custom environment
docker-compose -f docker-compose.test.yml --env-file .env.docker.custom up
```

## Troubleshooting Common Setup Issues

### Network Connectivity Issues

If tests can't connect to the local blockchain:

```bash
# Check if the node is running
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":67}' http://localhost:8545

# Restart the node
npx hardhat clean
npx hardhat node
```

### Dependency Conflicts

If you encounter dependency-related errors:

```bash
# Clear npm cache
npm cache clean --force

# Reinstall dependencies
rm -rf node_modules
npm install
```

### Insufficient Memory Issues

For tests requiring large amounts of memory:

```bash
# Increase Node.js memory limit
NODE_OPTIONS=--max_old_space_size=4096 npm test

# Or for specific test files
NODE_OPTIONS=--max_old_space_size=4096 npx hardhat test test/large-test.js
```

### Contract Compilation Errors

If you encounter contract compilation issues:

```bash
# Clean Hardhat artifacts
npx hardhat clean

# Try compiling with verbose output
npx hardhat compile --verbose
```

## Next Steps

Once you've successfully set up your testing environment:

1. Read the [Unit Testing](./testing-framework-unit-testing.md) guide to understand how to write effective tests
2. Explore the [Testing Tools](./testing-framework-tools.md) available in the framework
3. Review the [Best Practices](./testing-framework-best-practices.md) for writing maintainable tests

