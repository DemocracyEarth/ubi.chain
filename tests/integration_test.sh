#!/bin/bash

# UBI Chain Integration Test
# This script tests basic functionality of the UBI blockchain

set -e  # Exit on any error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting UBI Chain Integration Test${NC}"
echo

# Start a node in the background
echo -e "${YELLOW}Starting UBI Chain node...${NC}"
RUST_LOG=info cargo run --bin ubi-chain-node &
NODE_PID=$!

# Wait for node to start
echo -e "${YELLOW}Waiting for node to start...${NC}"
sleep 5

# Function to make RPC calls
rpc_call() {
    local method=$1
    local params=$2
    local endpoint=${3:-"http://127.0.0.1:9933"}
    
    curl -s -X POST -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" \
        $endpoint
}

# Test 1: Create accounts
echo -e "${YELLOW}Test 1: Creating accounts...${NC}"
ACCOUNT1="0x1234567890abcdef1234567890abcdef12345678"
ACCOUNT2="0xabcdef1234567890abcdef1234567890abcdef12"

RESULT1=$(rpc_call "create_account" "[$ACCOUNT1]")
RESULT2=$(rpc_call "create_account" "[$ACCOUNT2]")

if [[ $RESULT1 == *"error"* ]]; then
    echo -e "${RED}Failed to create account 1: $RESULT1${NC}"
    kill $NODE_PID
    exit 1
else
    echo -e "${GREEN}Account 1 created successfully${NC}"
fi

if [[ $RESULT2 == *"error"* ]]; then
    echo -e "${RED}Failed to create account 2: $RESULT2${NC}"
    kill $NODE_PID
    exit 1
else
    echo -e "${GREEN}Account 2 created successfully${NC}"
fi

# Test 2: Verify accounts
echo -e "${YELLOW}Test 2: Verifying accounts...${NC}"
RESULT1=$(rpc_call "verify_account" "[$ACCOUNT1]")
RESULT2=$(rpc_call "verify_account" "[$ACCOUNT2]")

if [[ $RESULT1 == *"error"* ]]; then
    echo -e "${RED}Failed to verify account 1: $RESULT1${NC}"
    kill $NODE_PID
    exit 1
else
    echo -e "${GREEN}Account 1 verified successfully${NC}"
fi

if [[ $RESULT2 == *"error"* ]]; then
    echo -e "${RED}Failed to verify account 2: $RESULT2${NC}"
    kill $NODE_PID
    exit 1
else
    echo -e "${GREEN}Account 2 verified successfully${NC}"
fi

# Test 3: Wait for UBI accrual
echo -e "${YELLOW}Test 3: Waiting for UBI accrual...${NC}"
echo -e "${YELLOW}Sleeping for 10 seconds to allow UBI to accrue...${NC}"
sleep 10

# Test 4: Check account balances
echo -e "${YELLOW}Test 4: Checking account balances...${NC}"
RESULT1=$(rpc_call "get_account_info" "[$ACCOUNT1]")
RESULT2=$(rpc_call "get_account_info" "[$ACCOUNT2]")

echo -e "${GREEN}Account 1 info: $RESULT1${NC}"
echo -e "${GREEN}Account 2 info: $RESULT2${NC}"

# Test 5: Submit a transaction
echo -e "${YELLOW}Test 5: Submitting a transaction...${NC}"
TX_PARAMS="[{\"from\":\"$ACCOUNT1\",\"to\":\"$ACCOUNT2\",\"amount\":10,\"fee\":1}]"
RESULT=$(rpc_call "submit_transaction" "$TX_PARAMS")

if [[ $RESULT == *"error"* ]]; then
    echo -e "${RED}Failed to submit transaction: $RESULT${NC}"
    kill $NODE_PID
    exit 1
else
    echo -e "${GREEN}Transaction submitted successfully: $RESULT${NC}"
fi

# Test 6: Wait for transaction to be processed
echo -e "${YELLOW}Test 6: Waiting for transaction to be processed...${NC}"
echo -e "${YELLOW}Sleeping for 5 seconds to allow transaction to be processed...${NC}"
sleep 5

# Test 7: Check account balances after transaction
echo -e "${YELLOW}Test 7: Checking account balances after transaction...${NC}"
RESULT1=$(rpc_call "get_account_info" "[$ACCOUNT1]")
RESULT2=$(rpc_call "get_account_info" "[$ACCOUNT2]")

echo -e "${GREEN}Account 1 info after transaction: $RESULT1${NC}"
echo -e "${GREEN}Account 2 info after transaction: $RESULT2${NC}"

# Test 8: Test Ethereum compatibility
echo -e "${YELLOW}Test 8: Testing Ethereum compatibility...${NC}"
ETH_RESULT=$(rpc_call "eth_blockNumber" "[]" "http://127.0.0.1:8545")
echo -e "${GREEN}Ethereum block number: $ETH_RESULT${NC}"

# Clean up
echo -e "${YELLOW}Cleaning up...${NC}"
kill $NODE_PID

echo
echo -e "${GREEN}Integration test completed successfully!${NC}" 