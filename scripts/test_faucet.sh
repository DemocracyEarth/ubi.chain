#!/bin/bash

# Test script for the UBI Chain faucet
# This script helps diagnose issues with the faucet and transaction processing

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}UBI Chain - Faucet Test Script${NC}"
echo "This script will test the faucet functionality and transaction processing."
echo

# Get the recipient address
if [ -z "$1" ]; then
    echo -e "${YELLOW}Please enter your MetaMask wallet address:${NC}"
    read ADDRESS
else
    ADDRESS=$1
fi

# Validate the address format
if [[ ! $ADDRESS =~ ^0x[a-fA-F0-9]{40}$ ]]; then
    echo -e "${RED}Invalid Ethereum address format. Address should start with '0x' followed by 40 hexadecimal characters.${NC}"
    exit 1
fi

echo -e "${YELLOW}Testing with address: ${ADDRESS}${NC}"
echo

# Step 1: Check if the node is running
echo -e "${YELLOW}Step 1: Checking if the UBI Chain node is running...${NC}"
if ! curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://localhost:8545 > /dev/null; then
    echo -e "${RED}UBI Chain node is not running or not accessible at http://localhost:8545${NC}"
    echo -e "${YELLOW}Please start the node with:${NC}"
    echo -e "RUST_LOG=info cargo run --bin ubi-chain-node"
    exit 1
fi
echo -e "${GREEN}UBI Chain node is running!${NC}"
echo

# Step 2: Check initial balance
echo -e "${YELLOW}Step 2: Checking initial balance...${NC}"
BALANCE_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$ADDRESS\", \"latest\"],\"id\":1}" http://localhost:8545)
BALANCE_HEX=$(echo $BALANCE_RESPONSE | grep -o '"result":"0x[a-fA-F0-9]*"' | cut -d'"' -f4)

if [ -z "$BALANCE_HEX" ]; then
    echo -e "${RED}Failed to get balance. Response: $BALANCE_RESPONSE${NC}"
else
    # Convert hex to decimal
    BALANCE_WEI=$(printf "%d" $BALANCE_HEX 2>/dev/null || echo "0")
    BALANCE_UBI=$(echo "scale=18; $BALANCE_WEI / 1000000000000000000" | bc)
    echo -e "${GREEN}Initial balance: $BALANCE_UBI UBI${NC}"
fi
echo

# Step 3: Request tokens from faucet
echo -e "${YELLOW}Step 3: Requesting 100 tokens from faucet...${NC}"
FAUCET_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"ubi_requestFromFaucet\",\"params\":[\"$ADDRESS\", 100],\"id\":1}" http://localhost:8545)
echo -e "${GREEN}Faucet response: $FAUCET_RESPONSE${NC}"
echo

# Step 4: Wait for transaction to be processed
echo -e "${YELLOW}Step 4: Waiting for transaction to be processed...${NC}"
echo -e "${YELLOW}Sleeping for 5 seconds...${NC}"
sleep 5
echo

# Step 5: Check new balance
echo -e "${YELLOW}Step 5: Checking new balance...${NC}"
NEW_BALANCE_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$ADDRESS\", \"latest\"],\"id\":1}" http://localhost:8545)
NEW_BALANCE_HEX=$(echo $NEW_BALANCE_RESPONSE | grep -o '"result":"0x[a-fA-F0-9]*"' | cut -d'"' -f4)

if [ -z "$NEW_BALANCE_HEX" ]; then
    echo -e "${RED}Failed to get new balance. Response: $NEW_BALANCE_RESPONSE${NC}"
else
    # Convert hex to decimal
    NEW_BALANCE_WEI=$(printf "%d" $NEW_BALANCE_HEX 2>/dev/null || echo "0")
    NEW_BALANCE_UBI=$(echo "scale=18; $NEW_BALANCE_WEI / 1000000000000000000" | bc)
    echo -e "${GREEN}New balance: $NEW_BALANCE_UBI UBI${NC}"
    
    # Check if balance increased
    if (( $(echo "$NEW_BALANCE_UBI > $BALANCE_UBI" | bc -l) )); then
        echo -e "${GREEN}Success! Balance increased by $(echo "$NEW_BALANCE_UBI - $BALANCE_UBI" | bc) UBI${NC}"
    else
        echo -e "${RED}Balance did not increase. Transaction may not have been processed correctly.${NC}"
    fi
fi
echo

# Step 6: Check recent blocks
echo -e "${YELLOW}Step 6: Checking recent blocks for transactions...${NC}"
BLOCK_NUMBER_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://localhost:8545)
BLOCK_NUMBER_HEX=$(echo $BLOCK_NUMBER_RESPONSE | grep -o '"result":"0x[a-fA-F0-9]*"' | cut -d'"' -f4)

if [ -z "$BLOCK_NUMBER_HEX" ]; then
    echo -e "${RED}Failed to get block number. Response: $BLOCK_NUMBER_RESPONSE${NC}"
else
    BLOCK_NUMBER=$(printf "%d" $BLOCK_NUMBER_HEX)
    echo -e "${GREEN}Current block number: $BLOCK_NUMBER${NC}"
    
    # Get the latest block
    LATEST_BLOCK_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"$BLOCK_NUMBER_HEX\", true],\"id\":1}" http://localhost:8545)
    echo -e "${GREEN}Latest block: $LATEST_BLOCK_RESPONSE${NC}"
fi
echo

echo -e "${GREEN}Test completed!${NC}"
echo -e "If you're still having issues, please check the node logs for more information."
echo -e "You can also try the balance checker tool at: http://localhost:8000/docs/tutorials/balance_checker.html" 