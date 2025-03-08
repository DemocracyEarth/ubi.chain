# Getting Started with UBI Chain Node

## Prerequisites

- Rust toolchain (latest stable)
- Git
- Unix-like operating system (Linux/MacOS)

## Installation

1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

2. Clone and build:
```bash
git clone https://github.com/yourusername/ubi-chain.git
cd ubi-chain
cargo build --release
```

## Running a Node

### Basic Usage

Start a standalone node:
```bash
cargo run --release --bin ubi-chain-node -- --port 30333
```

Enable verbose logging:
```bash
RUST_LOG=debug cargo run --release --bin ubi-chain-node -- --port 30333
```

### Running Multiple Nodes

1. First node (Terminal 1):
```bash
cargo run --release --bin ubi-chain-node -- --port 30333
```

2. Second node (Terminal 2):
```bash
cargo run --release --bin ubi-chain-node -- --port 30334 --peers 127.0.0.1:30333
```

3. Third node (Terminal 3):
```bash
cargo run --release --bin ubi-chain-node -- --port 30335 --peers 127.0.0.1:30333,127.0.0.1:30334
```

## Configuration Options

- `--port`: P2P network port (default: 30333)
- `--p2p-host`: P2P network host (default: 127.0.0.1)
- `--peers`: Comma-separated list of peer addresses
- `--rpc-host`: RPC server host (default: 127.0.0.1)
- `--rpc-port`: RPC server port (default: P2P port - 20400)

## Logging Levels

Set logging verbosity using RUST_LOG:
```bash
RUST_LOG=trace   # Most verbose
RUST_LOG=debug   # Detailed debugging
RUST_LOG=info    # General information
RUST_LOG=warn    # Warnings only
RUST_LOG=error   # Errors only
```

Target specific components:
```bash
RUST_LOG=ubi_chain_node=debug,p2p=trace,rpc=info
```

## Troubleshooting

Common issues:
- Port already in use: Try a different port
- Connection refused: Ensure peer node is running
- RPC errors: Check if RPC port is available

## Next Steps

- Read the [Architecture Documentation](../architecture/ARCHITECTURE.md)
- Explore the [API Documentation](../api/API.md)
- Check the [Contributing Guide](../contributing/CONTRIBUTING.md) 