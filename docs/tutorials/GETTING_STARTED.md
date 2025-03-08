# Getting Started with UBI Chain

This guide will walk you through setting up and interacting with UBI Chain.

## Prerequisites

Before starting, ensure you have:
- Rust toolchain (latest stable)
- Git
- A Unix-like operating system (Linux/MacOS)
- At least 2GB of RAM
- 10GB of free disk space

## Installation Steps

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### 2. Clone the Repository
```bash
git clone https://github.com/yourusername/ubi-chain.git
cd ubi-chain
```

### 3. Build the Project
```bash
# Check dependencies
cargo check

# Build in release mode
cargo build --release
```

## Running a Node

### Single Node
```bash
cargo run --release --bin ubi-chain-node
```

### Running Multiple Nodes Locally
To test peer-to-peer functionality locally, you can run multiple nodes with different port mappings:

```bash
# Terminal 1 - First node on default ports
cargo run --release --bin ubi-chain-node

# Terminal 2 - Second node with different ports
cargo run --release --bin ubi-chain-node -p 30334 --ws-port 9945 --rpc-port 9934

# Terminal 3 - Third node with different ports
cargo run --release --bin ubi-chain-node -p 30335 --ws-port 9946 --rpc-port 9935
```

Each node needs unique ports for:
- P2P communication (-p or --port-mapping)
- WebSocket (--ws-port)
- RPC (--rpc-port)

### Production Mode
```bash
cargo run --release --bin ubi-chain-node \
  --base-path /data/ubi-chain \
  --chain mainnet \
  --name my-node \
  -p 30333 \
  --ws-port 9944 \
  --rpc-port 9933
```

## Interacting with the Chain

### 1. Using Web Interface

1. Open https://polkadot.js.org/apps/
2. Click on the network selector (top left)
3. Choose "Development" or enter your node's WebSocket URL
4. You can now:
   - View account balances
   - Submit transactions
   - Monitor network status
   - View blockchain events

### 2. Using RPC API

Example using curl:
```bash
# Query chain state
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getBlock", "params":[]}' \
  http://localhost:9933

# Get account balance
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "state_getBalance", "params":["ADDRESS"]}' \
  http://localhost:9933
```

### 3. Using WebSocket

Example using Node.js:
```javascript
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:9944');

ws.on('open', () => {
  // Subscribe to new blocks
  ws.send(JSON.stringify({
    id: 1,
    jsonrpc: '2.0',
    method: 'chain_subscribeNewHeads',
    params: []
  }));
});

ws.on('message', (data) => {
  console.log('New block:', JSON.parse(data));
});
```

## Common Operations

### Verify Your Identity
1. Gather required verification documents
2. Submit verification request
3. Wait for verification process
4. Check verification status

### Claim UBI
1. Ensure your identity is verified
2. Check claim period
3. Submit claim transaction
4. Monitor transaction status

### Request AI Resources
1. Check available resource pool
2. Submit resource request
3. Monitor allocation status
4. Use allocated resources

## Troubleshooting

### Common Issues

1. Node won't start
   - Check ports are available
   - Verify sufficient disk space
   - Check log files

2. Connection issues
   - Verify WebSocket endpoint
   - Check firewall settings
   - Ensure correct ports are open

3. Transaction failures
   - Check account balance
   - Verify nonce
   - Check transaction parameters

### Debug Tools

```bash
# Check node status
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  http://localhost:9933

# View logs
tail -f /data/ubi-chain/node.log
```

## Next Steps

- Read the [Architecture Documentation](../architecture/ARCHITECTURE.md)
- Explore the [API Documentation](../api/API.md)
- Join the community channels
- Consider contributing to the project 