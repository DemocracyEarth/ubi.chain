# UBI Chain

A Human-Verified Network for Democratic AI Access and Universal Basic Income.

## Overview

UBI Chain is a blockchain project designed to provide a universal basic income (UBI) to verified human participants, while also enabling democratic access to AI resources.

## Project Structure

- **node**: The main blockchain node implementation (`ubi-chain-node`)
- **runtime**: Core blockchain logic and state management (`ubi-chain-runtime`)
- **rpc**: JSON-RPC interface for interacting with the blockchain (`ubi-chain-rpc`)

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
   cargo run --release --bin ubi-chain-node -- --dev
   ```

2. Start a node with custom configuration:
   ```bash
   cargo run --release --bin ubi-chain-node -- \
     --base-path /tmp/node01 \
     --chain local \
     --port 30333 \
     --ws-port 9945 \
     --rpc-port 9933
   ```

### Interacting with the Chain

1. Using the RPC Interface:
   The RPC interface is provided by the `ubi-chain-rpc` package. You can interact with it using standard HTTP/WebSocket clients.

2. Using Web Interface:
   - Open https://polkadot.js.org/apps/
   - Click on the network selector (top left)
   - Choose "Development" or enter custom endpoint: ws://127.0.0.1:9944

### Common Operations

- Check node status:
  ```bash
  curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://localhost:9933
  ```

- View logs:
  ```bash
  tail -f /tmp/node01/log/node.log
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

- If you encounter build errors, try:
  ```bash
  cargo clean
  cargo update
  cargo build --release
  ```

- For node connection issues:
  1. Check if ports are available
  2. Ensure firewall settings allow connections
  3. Verify WebSocket endpoint is accessible

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or feedback, please contact us at [@ubidoteth](http://x.com/ubidoteth).
