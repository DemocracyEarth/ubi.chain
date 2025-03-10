#!/bin/bash

# Build the node
cargo build

# Run the node with logging
RUST_LOG=info cargo run --bin ubi-chain-node

# The script will exit when the node exits 