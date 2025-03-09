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

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update stable
  ```

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

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test -p node
```

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
