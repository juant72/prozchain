# Testing in CI Environments

## Overview

Tests that run perfectly in a developer's local environment may behave differently in a CI environment. This chapter explores how to optimize tests for CI environments, handle test failures, and effectively visualize test results to maximize the value of automated testing in blockchain applications.

## Optimizing Tests for CI

### Test Splitting and Parallelization

Strategies for distributing test workloads:

```yaml
jobs:
  split-tests:
    runs-on: ubuntu-latest
    
    strategy:
      fail-fast: false
      matrix:
        shard: [1, 2, 3, 4]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      
      - name: Install Dependencies
        run: npm ci
      
      - name: Run Tests (Shard ${{ matrix.shard }}/4)
        run: npm run test -- --shard=${{ matrix.shard }}/4
```

Here's a sample implementation of a test sharding script:

```javascript
// scripts/test-sharding.js
const Mocha = require('mocha');
const glob = require('glob');

// Get all test files
const files = glob.sync('test/**/*.test.js');

// Parse shard argument (e.g., "1/4" means first shard out of four)
const shardArg = process.argv[2] || '1/1';
const [shardIndex, totalShards] = shardArg.split('/').map(Number);

if (shardIndex < 1 || shardIndex > totalShards || isNaN(shardIndex) || isNaN(totalShards)) {
  console.error('Invalid shard argument. Use format "N/M" where N <= M');
  process.exit(1);
}

// Calculate which files belong to this shard
const filesPerShard = Math.ceil(files.length / totalShards);
const startIdx = (shardIndex - 1) * filesPerShard;
const endIdx = Math.min(startIdx + filesPerShard, files.length);
const shardFiles = files.slice(startIdx, endIdx);

console.log(`Running shard ${shardIndex}/${totalShards} with ${shardFiles.length} test files`);

// Run tests for this shard
const mocha = new Mocha();
shardFiles.forEach(file => mocha.addFile(file));

mocha.run(failures => {
  process.exitCode = failures ? 1 : 0;
});
```

### Conditional Testing Based on Changes

Running only relevant tests based on code changes:

```yaml
jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      contracts: ${{ steps.filter.outputs.contracts }}
      frontend: ${{ steps.filter.outputs.frontend }}
      backend: ${{ steps.filter.outputs.backend }}
    
    steps:
      - uses: actions/checkout@v3
      
      - uses: dorny/paths-filter@v2
        id: filter
        with:
          filters: |
            contracts:
              - 'contracts/**'
              - 'test/**'
              - 'hardhat.config.js'
            frontend:
              - 'frontend/**'
            backend:
              - 'backend/**'
  
  contract-tests:
    needs: detect-changes
    if: ${{ needs.detect-changes.outputs.contracts == 'true' }}
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
      - name: Install Dependencies
        run: npm ci
      - name: Run Contract Tests
        run: npm run test:contracts
  
  frontend-tests:
    needs: detect-changes
    if: ${{ needs.detect-changes.outputs.frontend == 'true' }}
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
      - name: Install Frontend Dependencies
        run: cd frontend && npm ci
      - name: Run Frontend Tests
        run: cd frontend && npm test
```

A sample selective testing script:

```javascript
// scripts/selective-tests.js
const { execSync } = require('child_process');
const fs = require('fs');

// Get changed files from git
const changedFiles = execSync('git diff --name-only HEAD~1 HEAD')
  .toString()
  .trim()
  .split('\n');

const runAllTests = process.env.CI_FORCE_ALL_TESTS === 'true';

// Determine affected test categories
let shouldRunContractTests = runAllTests;
let shouldRunFrontendTests = runAllTests;
let shouldRunBackendTests = runAllTests;

changedFiles.forEach(file => {
  if (file.startsWith('contracts/') || file.startsWith('test/contracts/')) {
    shouldRunContractTests = true;
  } else if (file.startsWith('frontend/')) {
    shouldRunFrontendTests = true;
  } else if (file.startsWith('backend/')) {
    shouldRunBackendTests = true;
  } else if (file === 'hardhat.config.js' || file === 'package.json' || file === 'package-lock.json') {
    // Configuration changes might affect everything
    shouldRunContractTests = true;
    shouldRunFrontendTests = true;
    shouldRunBackendTests = true;
  }
});

// Run tests selectively
if (shouldRunContractTests) {
  console.log('Running contract tests...');
  execSync('npm run test:contracts', { stdio: 'inherit' });
}

if (shouldRunFrontendTests) {
  console.log('Running frontend tests...');
  execSync('cd frontend && npm test', { stdio: 'inherit' });
}

if (shouldRunBackendTests) {
  console.log('Running backend tests...');
  execSync('cd backend && npm test', { stdio: 'inherit' });
}
```

### Test Timeouts and Retries

Handling flaky tests and long-running operations:

```yaml
steps:
  - name: Run Flaky Tests with Retries
    uses: nick-invision/retry@v2
    with:
      timeout_minutes: 10
      max_attempts: 3
      retry_on: error
      command: npm run test:flaky
  
  - name: Run Long-Running Tests with Extended Timeout
    timeout-minutes: 30
    run: npm run test:long-running
```

Configure Mocha to use retries for specific tests:

```javascript
// test/flaky-tests/network-conditions.test.js
describe('Tests with network conditions', function() {
  // Configure retries for this suite
  this.retries(3);
  
  it('should handle network latency', async function() {
    // This test might be flaky and will be retried up to 3 times
    // ... test implementation ...
  });
});
```

## Handling Test Failures

### Failure Notification System

Setting up alerts for test failures:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      # ... test steps ...
    
    if: ${{ failure() }}
    steps:
      - name: Send Slack Notification on Failure
        uses: slackapi/slack-github-action@v1.23.0
        with:
          payload: |
            {
              "text": "❌ CI Pipeline failed in ${{ github.repository }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*CI Pipeline Failed*\n*Repository:* ${{ github.repository }}\n*Branch:* ${{ github.ref_name }}\n*Commit:* ${{ github.sha }}\n*Commit Message:* ${{ github.event.head_commit.message }}\n*Author:* ${{ github.actor }}"
                  }
                },
                {
                  "type": "actions",
                  "elements": [
                    {
                      "type": "button",
                      "text": {
                        "type": "plain_text",
                        "text": "View Workflow"
                      },
                      "url": "https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}"
                    }
                  ]
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK
```

### Test Artifacts Collection

Gathering debug information when tests fail:

```yaml
steps:
  - name: Run Tests with Logger
    run: npm test > test-output.log || { cat test-output.log; exit 1; }
  
  - name: Collect Test Artifacts on Failure
    if: ${{ failure() }}
    run: |
      mkdir -p ./test-artifacts
      cp test-output.log ./test-artifacts/
      cp -r ./hardhat/cache/console-logs ./test-artifacts/ || true
      cp ./ganache.log ./test-artifacts/ || true
      cp -r ./screenshots ./test-artifacts/ || true
  
  - name: Upload Test Artifacts
    if: ${{ always() }}
    uses: actions/upload-artifact@v3
    with:
      name: test-artifacts
      path: ./test-artifacts
      retention-days: 7
```

### Test Results Visualization

Creating readable test reports:

```yaml
steps:
  - name: Run Tests with JUnit Reporter
    run: npm test -- --reporter mocha-junit-reporter
  
  - name: Publish Test Results
    uses: EnricoMi/publish-unit-test-result-action@v2
    if: ${{ always() }}
    with:
      files: |
        test-results/**/*.xml
        junit-*.xml
  
  - name: Generate HTML Test Report
    if: ${{ always() }}
    run: npx mocha-html-reporter -o test-report.html
  
  - name: Upload HTML Report
    if: ${{ always() }}
    uses: actions/upload-artifact@v3
    with:
      name: test-report
      path: test-report.html
```

Example configuration for generating HTML reports:

```javascript
// .mocharc.js
module.exports = {
  reporter: 'mochawesome',
  'reporter-option': [
    'reportDir=test-results',
    'reportFilename=test-report',
    'html=true',
    'json=true'
  ],
  timeout: 30000
};
```

## Test Analysis Automation

### Automated Issue Classification

Using AI to categorize test failures:

```yaml
- name: Analyze Test Failures
  if: ${{ failure() }}
  uses: actions/github-script@v6
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}
    script: |
      const fs = require('fs');
      
      // Read test output
      const testOutput = fs.readFileSync('test-output.log', 'utf8');
      
      // Define patterns for common issues
      const patterns = [
        { type: 'Network', regex: /network error|connection refused|timeout/i },
        { type: 'Gas', regex: /out of gas|exceed(ed)? gas limit/i },
        { type: 'Assertion', regex: /assertion( failed)?|expect(ed)?.*to( be)? |assert\./i },
        { type: 'Contract Error', regex: /revert(ed)?|require\(|invalid opcode/i }
      ];
      
      // Classify the error
      let errorType = 'Unknown';
      for (const pattern of patterns) {
        if (pattern.regex.test(testOutput)) {
          errorType = pattern.type;
          break;
        }
      }
      
      // Create or update comment on PR
      const context = github.context;
      if (context.payload.pull_request) {
        github.rest.issues.createComment({
          issue_number: context.payload.pull_request.number,
          owner: context.repo.owner,
          repo: context.repo.repo,
          body: `## Test Failure Analysis\n\nTests failed with error type: **${errorType}**\n\nPlease check the [workflow run](https://github.com/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}) for details.`
        });
      }
```

### Historical Test Analysis

Tracking test performance over time:

```javascript
// scripts/analyze-test-history.js
const fs = require('fs');
const path = require('path');

// Read current test results
const currentResults = require('../test-results/test-report.json');

// Path to historical data
const historyFile = path.join(__dirname, '../test-history.json');
let history = [];

// Load existing history if available
if (fs.existsSync(historyFile)) {
  history = JSON.parse(fs.readFileSync(historyFile, 'utf8'));
}

// Add current results to history
history.push({
  timestamp: new Date().toISOString(),
  commit: process.env.GITHUB_SHA || 'local',
  totalTests: currentResults.stats.tests,
  passed: currentResults.stats.passes,
  failed: currentResults.stats.failures,
  duration: currentResults.stats.duration,
  slowest: findSlowestTest(currentResults)
});

// Keep only last 100 entries
if (history.length > 100) {
  history = history.slice(-100);
}

// Write updated history
fs.writeFileSync(historyFile, JSON.stringify(history, null, 2));

// Generate trends report
generateTrendsReport(history);

// Helper functions
function findSlowestTest(results) {
  let slowest = { duration: 0 };
  
  function processSuite(suite) {
    suite.tests.forEach(test => {
      if (test.duration > slowest.duration) {
        slowest = {
          title: test.title,
          duration: test.duration
        };
      }
    });
    
    suite.suites.forEach(processSuite);
  }
  
  results.results.forEach(processSuite);
  return slowest;
}

function generateTrendsReport(history) {
  // Calculate trends
  const recentRuns = history.slice(-10);
  const avgDuration = recentRuns.reduce((sum, run) => sum + run.duration, 0) / recentRuns.length;
  const passRate = recentRuns.reduce((sum, run) => sum + (run.passed / run.totalTests * 100), 0) / recentRuns.length;
  
  // Generate report
  const report = {
    avgDuration,
    passRate,
    totalRuns: history.length,
    improvement: history.length > 1 ? 
      (history[history.length - 1].duration - history[0].duration) / history[0].duration * 100 : 0
  };
  
  fs.writeFileSync(
    path.join(__dirname, '../test-results/trends-report.json'), 
    JSON.stringify(report, null, 2)
  );
}
```

## Conclusion

Optimizing tests for CI environments is essential for efficient development workflows. By implementing proper test splitting, failure notification systems, and results visualization, teams can quickly identify and address issues in their blockchain applications.

The strategies in this chapter help ensure that tests are reliable, informative, and efficient when running in continuous integration environments.
