#!/bin/bash

# Test script for Ethereum-compatible JSON-RPC server with HTTP and WebSocket support

# Build the server
echo "Building Ethereum-compatible JSON-RPC server..."
cargo build --bin eth_rpc_server

# Start the server in the background
echo "Starting Ethereum-compatible JSON-RPC server..."
./target/debug/eth_rpc_server 127.0.0.1:8545 127.0.0.1:8546 2030 &
SERVER_PID=$!

# Wait for the server to start
sleep 2

# Test HTTP endpoints
echo "Testing HTTP endpoints..."

# Test eth_chainId
echo "eth_chainId:"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://127.0.0.1:8545

echo -e "\n"

# Test eth_blockNumber
echo "eth_blockNumber:"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://127.0.0.1:8545

echo -e "\n"

# Create an account
echo "Creating account..."
ACCOUNT="0x$(openssl rand -hex 20)"
echo "Account: $ACCOUNT"

# Test eth_getBalance
echo "eth_getBalance (before):"
curl -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$ACCOUNT\", \"latest\"],\"id\":1}" http://127.0.0.1:8545

echo -e "\n"

# Request from faucet
echo "Requesting from faucet:"
curl -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"ubi_requestFromFaucet\",\"params\":[\"$ACCOUNT\", 100],\"id\":1}" http://127.0.0.1:8545

echo -e "\n"

# Test eth_getBalance again
echo "eth_getBalance (after):"
curl -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$ACCOUNT\", \"latest\"],\"id\":1}" http://127.0.0.1:8545

echo -e "\n"

# Test eth_blockNumber again
echo "eth_blockNumber (after transaction):"
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://127.0.0.1:8545

echo -e "\n"

# Test WebSocket subscription (requires wscat)
if command -v wscat &> /dev/null; then
    echo "Testing WebSocket subscription..."
    echo "Subscribing to newHeads (in a separate terminal, run: wscat -c ws://127.0.0.1:8546 -x '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"]}')"
    
    # Create a second account
    ACCOUNT2="0x$(openssl rand -hex 20)"
    echo "Second account: $ACCOUNT2"
    
    # Send a transaction to trigger a new block
    echo "Sending a transaction to trigger a new block:"
    curl -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendTransaction\",\"params\":[{\"from\":\"$ACCOUNT\",\"to\":\"$ACCOUNT2\",\"value\":\"0x1\"}],\"id\":1}" http://127.0.0.1:8545
    
    echo -e "\n"
    echo "If you have a WebSocket client connected, you should see a notification for the new block."
else
    echo "wscat not found. Install it with: npm install -g wscat"
fi

# Clean up
echo "Stopping server..."
kill $SERVER_PID

echo "Done!" 