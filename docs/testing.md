# UBI Chain Testing Guide

This document provides comprehensive guidance on testing the UBI blockchain. It covers various testing approaches, from unit tests to integration tests and manual testing procedures.

## Table of Contents

- [Running All Tests](#running-all-tests)
- [Unit Testing](#unit-testing)
- [Running a Local Node](#running-a-local-node)
- [Multi-Node Testing](#multi-node-testing)
- [RPC API Testing](#rpc-api-testing)
- [Ethereum Compatibility Testing](#ethereum-compatibility-testing)
- [Transaction Testing](#transaction-testing)
- [UBI Distribution Testing](#ubi-distribution-testing)
- [State Persistence Testing](#state-persistence-testing)
- [Performance Testing](#performance-testing)
- [Integration Testing](#integration-testing)
- [Security Testing](#security-testing)
- [Troubleshooting](#troubleshooting)

## Running All Tests

We provide a convenient script to run all tests in the UBI Chain project:

```bash
./run_tests.sh
```

This script:
- Runs unit tests for all packages (runtime, rpc, node)
- Checks code formatting
- Runs clippy for code quality
- Executes integration tests (if available)
- Runs documentation tests
- Provides a summary of test results

## Unit Testing

Unit tests verify the core functionality of individual components:

```bash
# Test the runtime module (core blockchain logic)
cargo test -p ubi-chain-runtime

# Test the RPC module (API functionality)
cargo test -p ubi-chain-rpc

# Test the node module
cargo test -p ubi-chain-node
```

The runtime tests cover essential functionality like:
- Account creation and verification
- Balance management
- UBI distribution
- Transaction processing
- Fee distribution
- Merkle tree implementation for state verification
- Checkpoint creation and loading

## Running a Local Node

Start a local node for testing:

```bash
RUST_LOG=info cargo run --bin ubi-chain-node
```

This starts:
- A P2P network on 127.0.0.1:30333
- An RPC server on 127.0.0.1:9933
- An Ethereum-compatible JSON-RPC server on 127.0.0.1:8545
- Block production with a 1000ms block time

> **Note**: Always set the `RUST_LOG` environment variable when running the node to ensure proper logging and to avoid potential runtime issues.

## Multi-Node Testing

Test a multi-node setup by running multiple instances with different ports:

```bash
# Terminal 1 - First node
RUST_LOG=info cargo run --bin ubi-chain-node -- --port 30333 --rpc-port 9933

# Terminal 2 - Second node connecting to the first
RUST_LOG=info cargo run --bin ubi-chain-node -- --port 30334 --rpc-port 9934 --peers 127.0.0.1:30333
```

This tests:
- Peer discovery
- Block propagation
- Consensus mechanisms
- Network resilience

## RPC API Testing

Interact with the node using the JSON-RPC API:

```bash
# Example: Get account information
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account_info","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933

# Example: Create a new account
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"create_account","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933
```

Available RPC methods include:
- `get_account_info`: Retrieve account details
- `create_account`: Create a new account
- `submit_transaction`: Submit a transaction
- `verify_account`: Mark an account as human-verified
- `get_block`: Get block information
- `get_transaction`: Get transaction details
- `get_pending_transactions`: List pending transactions
- `get_network_status`: Get network information

## Ethereum Compatibility Testing

Test Ethereum compatibility using standard Ethereum tools:

```bash
# Using curl
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://127.0.0.1:8545

# Using web3.js
const Web3 = require('web3');
const web3 = new Web3('http://127.0.0.1:8545');
web3.eth.getBlockNumber().then(console.log);
```

Supported Ethereum JSON-RPC methods:
- `eth_blockNumber`: Get current block number
- `eth_getBalance`: Get account balance
- `eth_sendTransaction`: Send a transaction
- `eth_getTransactionCount`: Get transaction count
- `eth_getBlockByNumber`: Get block by number
- `eth_getBlockByHash`: Get block by hash
- `eth_getTransactionByHash`: Get transaction by hash
- `eth_chainId`: Get chain ID

## Transaction Testing

Test transaction submission and processing:

```bash
# Submit a transaction via RPC
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit_transaction","params":[{"from":"0x1234567890abcdef1234567890abcdef12345678","to":"0xabcdef1234567890abcdef1234567890abcdef12","amount":100,"fee":1}],"id":1}' http://127.0.0.1:9933

# Check transaction status
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_transaction","params":["TRANSACTION_HASH"],"id":1}' http://127.0.0.1:9933
```

Test scenarios:
1. Valid transaction between accounts
2. Transaction with insufficient balance
3. Transaction with invalid signature
4. Transaction with zero fee
5. Transaction to non-existent account

## UBI Distribution Testing

Test the UBI distribution mechanism:

```bash
# Create an account
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"create_account","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933

# Verify the account
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"verify_account","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933

# Check balance after some time to see UBI accrual
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account_info","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933
```

Test scenarios:
1. UBI accrual for verified accounts
2. No UBI accrual for unverified accounts
3. UBI accrual rate over time
4. UBI distribution after node restart

## State Persistence Testing

Test checkpoint creation and loading:

```bash
# Create a checkpoint
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"create_checkpoint","params":[true],"id":1}' http://127.0.0.1:9933

# List checkpoints
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"list_checkpoints","params":[],"id":1}' http://127.0.0.1:9933

# Load a checkpoint
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"load_checkpoint","params":["checkpoint_filename"],"id":1}' http://127.0.0.1:9933
```

Test scenarios:
1. Create and load checkpoints
2. Verify state consistency after loading a checkpoint
3. Test automatic checkpoint creation
4. Test checkpoint loading during node startup

## Performance Testing

Test the performance of the blockchain:

```bash
# Example script to generate many transactions
for i in {1..1000}; do
  curl -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"submit_transaction\",\"params\":[{\"from\":\"0x1234567890abcdef1234567890abcdef12345678\",\"to\":\"0xabcdef1234567890abcdef1234567890abcdef12\",\"amount\":1,\"fee\":1}],\"id\":$i}" http://127.0.0.1:9933
done
```

Performance metrics to monitor:
1. Transactions per second (TPS)
2. Block production time
3. Memory usage
4. CPU utilization
5. Network bandwidth

## Integration Testing

Create integration tests that simulate real-world usage scenarios:

1. Account creation and verification
2. UBI distribution over time
3. Transactions between accounts
4. Fee collection and distribution
5. State persistence and recovery

Example integration test script:

```bash
#!/bin/bash

# Start a node
RUST_LOG=info cargo run --bin ubi-chain-node &
NODE_PID=$!

# Wait for node to start
sleep 5

# Create accounts
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"create_account","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"create_account","params":["0xabcdef1234567890abcdef1234567890abcdef12"],"id":1}' http://127.0.0.1:9933

# Verify accounts
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"verify_account","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"verify_account","params":["0xabcdef1234567890abcdef1234567890abcdef12"],"id":1}' http://127.0.0.1:9933

# Wait for UBI accrual
sleep 10

# Submit transaction
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit_transaction","params":[{"from":"0x1234567890abcdef1234567890abcdef12345678","to":"0xabcdef1234567890abcdef1234567890abcdef12","amount":10,"fee":1}],"id":1}' http://127.0.0.1:9933

# Check balances
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account_info","params":["0x1234567890abcdef1234567890abcdef12345678"],"id":1}' http://127.0.0.1:9933
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"get_account_info","params":["0xabcdef1234567890abcdef1234567890abcdef12"],"id":1}' http://127.0.0.1:9933

# Clean up
kill $NODE_PID
```

## Security Testing

Test the security aspects of the blockchain:

1. Try submitting invalid transactions:
   ```bash
   # Transaction with invalid signature
   curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit_transaction","params":[{"from":"0x1234567890abcdef1234567890abcdef12345678","to":"0xabcdef1234567890abcdef1234567890abcdef12","amount":100,"fee":1,"signature":"invalid"}],"id":1}' http://127.0.0.1:9933
   ```

2. Attempt to spend more than available balance:
   ```bash
   # Transaction with amount exceeding balance
   curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit_transaction","params":[{"from":"0x1234567890abcdef1234567890abcdef12345678","to":"0xabcdef1234567890abcdef1234567890abcdef12","amount":999999999,"fee":1}],"id":1}' http://127.0.0.1:9933
   ```

3. Test transaction replay protection:
   ```bash
   # Submit the same transaction twice
   TX='{"jsonrpc":"2.0","method":"submit_transaction","params":[{"from":"0x1234567890abcdef1234567890abcdef12345678","to":"0xabcdef1234567890abcdef1234567890abcdef12","amount":1,"fee":1}],"id":1}'
   curl -X POST -H "Content-Type: application/json" --data "$TX" http://127.0.0.1:9933
   curl -X POST -H "Content-Type: application/json" --data "$TX" http://127.0.0.1:9933
   ```

4. Verify signature validation:
   ```bash
   # Transaction with mismatched signature
   curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"submit_transaction","params":[{"from":"0x1234567890abcdef1234567890abcdef12345678","to":"0xabcdef1234567890abcdef1234567890abcdef12","amount":1,"fee":1,"signature":"0x9876543210abcdef9876543210abcdef98765432"}],"id":1}' http://127.0.0.1:9933
   ```

## Troubleshooting

Common testing issues and solutions:

1. **Node crashes with "Cannot drop a runtime in a context where blocking is not allowed"**:
   - Always set the `RUST_LOG` environment variable when running the node:
     ```bash
     RUST_LOG=info cargo run --bin ubi-chain-node
     ```

2. **Tests fail with "connection refused"**:
   - Ensure the node is running and listening on the expected ports
   - Check for port conflicts with other applications

3. **Transaction submission fails**:
   - Verify the account exists and has sufficient balance
   - Check that the transaction format is correct
   - Ensure the node is running and accepting transactions

4. **UBI not accruing**:
   - Verify the account is marked as human-verified
   - Allow sufficient time for UBI to accrue (based on the UBI_TOKENS_PER_HOUR rate)
   - Check that the node is producing blocks correctly

5. **Checkpoint loading fails**:
   - Verify the checkpoint file exists and is accessible
   - Check that the checkpoint format is compatible with the current node version
   - Ensure sufficient permissions to read the checkpoint file 