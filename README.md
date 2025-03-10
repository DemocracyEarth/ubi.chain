# UBI Chain

A Human-Verified Network for Democratic AI Access and Universal Basic Income.

## Overview

UBI Chain is a blockchain project designed to provide a universal basic income (UBI) to verified human participants, while also enabling democratic access to AI resources.

## Project Structure

- **node**: The main blockchain node implementation (`ubi-chain-node`)
- **runtime**: Core blockchain logic and state management (`ubi-chain-runtime`)
- **rpc**: JSON-RPC interface for interacting with the blockchain (`ubi-chain-rpc`)

## Features

- **Universal Basic Income**: Distributes UBI tokens to verified human participants
- **Human Verification**: Ensures only real humans can participate in the network
- **Ethereum Compatibility**: Connect with standard Ethereum wallets via JSON-RPC API
- **P2P Networking**: Decentralized node communication for robust network operation
- **Democratic AI Access**: Fair allocation of AI resources to all participants
- **Testnet with Faucet**: Request test tokens from the faucet service for development and testing

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update stable
  ```
- [MetaMask](https://metamask.io/) browser extension (for wallet integration)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/ubi-chain.git
   cd ubi-chain
   ```

2. Install dependencies:
   ```bash
   cargo check
   ```

### Building

Build all components of the project:
```bash
cargo build --release
```

### Running with MetaMask Support

To run the node with MetaMask support:

```bash
cargo run --bin ubi-chain-node -- --eth-rpc-host 127.0.0.1 --eth-rpc-port 8545 --chain-id 2030
```

This will start the node with Ethereum JSON-RPC compatibility enabled, allowing you to connect MetaMask to your local UBI Chain node.

### Connecting MetaMask

1. Open MetaMask and add a new network with the following details:
   - **Network Name**: UBI Chain Local
   - **RPC URL**: http://localhost:8545
   - **Chain ID**: 2030
   - **Currency Symbol**: UBI

2. For detailed instructions on using MetaMask with UBI Chain, see the [MetaMask Integration Tutorial](docs/tutorials/METAMASK_INTEGRATION.md).

3. You can also use our test script to quickly set up and test the MetaMask integration:
   ```bash
   ./scripts/test_metamask.sh
   ```
   This script will start the node with MetaMask support and open the integration page in your browser.

   If you encounter "ethereum is not defined" errors, try using the web server option:
   ```bash
   ./scripts/test_metamask.sh --server
   ```
   This will serve the files using a local web server, which often resolves MetaMask detection issues.

### Running the Testnet

Run the UBI Chain testnet node:
```bash
RUST_LOG=info cargo run --bin ubi-chain-node
```

This will start a local testnet node with the following services:
- P2P network on port 30333
- JSON-RPC server on port 9933
- Ethereum-compatible JSON-RPC server on port 8545

The testnet is designed for development and testing purposes. Unlike the mock transaction generation in previous versions, the testnet operates like a real blockchain but with a faucet service that allows developers to request test tokens.

### Using the Faucet

You can request test tokens from the faucet using the JSON-RPC API:

```bash
# Request 10 tokens (default amount)
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"requestFromFaucet","params":["0xYOUR_ADDRESS_HERE"],"id":1}' http://127.0.0.1:9933

# Request a specific amount of tokens (maximum 100)
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"requestFromFaucet","params":["0xYOUR_ADDRESS_HERE", 50],"id":1}' http://127.0.0.1:9933
```

The faucet is also available through the Ethereum-compatible JSON-RPC endpoint:

```bash
# Request tokens from the Ethereum-compatible endpoint
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"ubi_requestFromFaucet","params":["0xYOUR_ADDRESS_HERE", 50],"id":1}' http://127.0.0.1:8545
```

The faucet will:
1. Create your account if it doesn't exist
2. Send the requested amount of tokens to your address (up to 100 tokens per request)
3. Return your new balance

#### Faucet Response Format

The faucet returns a JSON response with the following structure:

```json
{
  "success": true,
  "amount": 50,
  "newBalance": 60,
  "address": "0xYOUR_ADDRESS_HERE"
}
```

If there's an error, the response will include an error message:

```json
{
  "success": false,
  "error": "Error message here"
}
```

### Running the Node

1. Start a development node:
   ```bash
   RUST_LOG=info cargo run --bin ubi-chain-node
   ```

2. Start a node with custom configuration:
   ```bash
   RUST_LOG=info cargo run --bin ubi-chain-node -- \
     --port 30333 \
     --p2p-host 127.0.0.1 \
     --rpc-host 127.0.0.1 \
     --chain-id 2030
   ```

3. Adjust logging level as needed:
   ```bash
   # For minimal logging (warnings and errors only)
   RUST_LOG=warn cargo run --bin ubi-chain-node
   
   # For standard information
   RUST_LOG=info cargo run --bin ubi-chain-node
   
   # For detailed debugging information
   RUST_LOG=debug cargo run --bin ubi-chain-node
   ```

> **Note**: Always set the `RUST_LOG` environment variable when running the node to ensure proper logging and to avoid potential runtime issues.

### Running Multiple Nodes

1. First node (Terminal 1):
   ```bash
   RUST_LOG=info cargo run --bin ubi-chain-node -- --port 30333
   ```

2. Second node (Terminal 2):
   ```bash
   RUST_LOG=info cargo run --bin ubi-chain-node -- --port 30334 --peers 127.0.0.1:30333
   ```

### Ethereum Compatibility

UBI Chain provides Ethereum JSON-RPC compatibility, allowing you to connect standard Ethereum wallets:

1. Start a node with Ethereum RPC enabled:
   ```bash
   RUST_LOG=info cargo run --bin ubi-chain-node -- --eth-rpc-port 8545 --chain-id 2030
   ```

2. Connect MetaMask or other Ethereum wallets:
   - Network Name: UBI Chain
   - RPC URL: http://localhost:8545
   - Chain ID: 2030
   - Currency Symbol: UBI

### Configuration Options

- `--port`: P2P network port (default: 30333)
- `--p2p-host`: P2P network host (default: 127.0.0.1)
- `--peers`: Comma-separated list of peer addresses
- `--rpc-host`: RPC server host (default: 127.0.0.1)
- `--rpc-port`: RPC server port (default: P2P port - 20400)
- `--eth-rpc-host`: Ethereum RPC host (default: 127.0.0.1)
- `--eth-rpc-port`: Ethereum RPC port (default: 8545)
- `--chain-id`: Chain ID for Ethereum compatibility (default: 2030)
- `--disable-eth-rpc`: Disable Ethereum JSON-RPC server

### Interacting with the Chain

1. Using the RPC Interface:
   The RPC interface is provided by the `ubi-chain-rpc` package. You can interact with it using standard HTTP/WebSocket clients.

2. Using Ethereum Wallets:
   - Open MetaMask or any Ethereum-compatible wallet
   - Add a custom network with Chain ID 2030
   - Connect to http://localhost:8545 (or your custom RPC endpoint)

### Common Operations

- Check node status:
  ```bash
  curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://localhost:9933
  ```

- View logs:
  ```bash
  RUST_LOG=debug cargo run --release --bin ubi-chain-node
  ```

## Development

### Running Tests

We provide a comprehensive testing framework for UBI Chain:

```bash
# Run all tests with the test script
./run_tests.sh

# Run specific package tests
cargo test -p ubi-chain-runtime
cargo test -p ubi-chain-rpc
cargo test -p ubi-chain-node
```

For detailed testing documentation, see [Testing Guide](docs/testing.md).

### Code Style

Before submitting PR, ensure code is formatted:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features
```

## Contributing

1. Fork the repository
2. Create a new branch: `git checkout -b feature/your-feature-name`
3. Make your changes and commit them: `git commit -m 'Add some feature'`
4. Push to the branch: `git push origin feature/your-feature-name`
5. Create a pull request

## Troubleshooting

- If you encounter a runtime panic with "Cannot drop a runtime in a context where blocking is not allowed":
  ```
  thread 'main' panicked at '...Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped from within an asynchronous context.'
  ```
  This is typically caused by not setting the `RUST_LOG` environment variable. Always run the node with:
  ```bash
  RUST_LOG=info cargo run --bin ubi-chain-node
  ```

- If you see "Failed to send block: channel closed" errors in the logs, this is expected in a test environment where no peers are connected to receive the blocks.

- If you encounter build errors, try:
  ```bash
  cargo clean
  cargo update
  cargo build
  ```

- For node connection issues:
  1. Check if ports are available
  2. Ensure firewall settings allow connections
  3. Verify WebSocket endpoint is accessible

- For Ethereum wallet connection issues:
  1. Verify the chain ID is set to 2030
  2. Ensure the RPC endpoint is accessible
  3. Check that the Ethereum RPC server is enabled

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or feedback, please contact us at [@ubidoteth](http://x.com/ubidoteth).

### Using the Testnet with MetaMask

The UBI Chain testnet is fully compatible with MetaMask and other Ethereum wallets. Here's how to set it up:

1. Start the testnet node:
   ```bash
   RUST_LOG=info cargo run --bin ubi-chain-node
   ```

2. Open MetaMask and add a new network with the following details:
   - **Network Name**: UBI Chain Testnet
   - **RPC URL**: http://localhost:8545
   - **Chain ID**: 2030
   - **Currency Symbol**: UBI

3. Request test tokens from the faucet:
   - Copy your MetaMask address
   - Use the faucet API to request tokens:
     ```bash
     curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"ubi_requestFromFaucet","params":["YOUR_METAMASK_ADDRESS", 100],"id":1}' http://127.0.0.1:8545
     ```
   - The tokens should appear in your MetaMask wallet

4. You can now use MetaMask to:
   - Send transactions to other addresses
   - Interact with smart contracts (if implemented)
   - Sign messages and transactions

#### Testnet Limitations

The current testnet implementation has the following limitations:
- No mining or staking mechanism (blocks are produced automatically)
- Limited transaction types (only token transfers)
- No smart contract execution (planned for future versions)
- Simplified consensus mechanism
