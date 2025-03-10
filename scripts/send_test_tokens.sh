#!/bin/bash

# Send test tokens to a MetaMask wallet address
# This script helps users get test tokens for the UBI Chain

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
AMOUNT=100
FROM_ADDRESS="0x1111111111111111111111111111111111111111"

echo -e "${GREEN}UBI Chain - Send Test Tokens${NC}"
echo "This script will send test tokens to your MetaMask wallet address."
echo

# Get the recipient address
if [ -z "$1" ]; then
    echo -e "${YELLOW}Please enter your MetaMask wallet address:${NC}"
    read TO_ADDRESS
else
    TO_ADDRESS=$1
fi

# Validate the address format
if [[ ! $TO_ADDRESS =~ ^0x[a-fA-F0-9]{40}$ ]]; then
    echo -e "${RED}Invalid Ethereum address format. Address should start with '0x' followed by 40 hexadecimal characters.${NC}"
    exit 1
fi

echo -e "${YELLOW}Sending $AMOUNT UBI tokens to $TO_ADDRESS...${NC}"

# Check if the node is running
if ! pgrep -f "ubi-chain-node" > /dev/null; then
    echo -e "${YELLOW}UBI Chain node is not running. Starting it now...${NC}"
    
    # Start the node in the background
    cargo run --bin ubi-chain-node -- --eth-rpc-host 127.0.0.1 --eth-rpc-port 8545 --chain-id 2030 > node_output.log 2>&1 &
    
    # Wait for the node to start
    sleep 5
    
    if ! pgrep -f "ubi-chain-node" > /dev/null; then
        echo -e "${RED}Failed to start the node. Check node_output.log for details.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Node started successfully!${NC}"
fi

# Create a temporary JSON file for the RPC request
TMP_FILE=$(mktemp)
cat > $TMP_FILE << EOF
{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
        "from": "$FROM_ADDRESS",
        "to": "$TO_ADDRESS",
        "value": "0x$(printf '%x' $(($AMOUNT * 1000000000000000000)))",
        "gas": "0x5208",
        "gasPrice": "0x3b9aca00"
    }],
    "id": 1
}
EOF

# Send the transaction
echo -e "${YELLOW}Sending transaction...${NC}"
RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" --data @$TMP_FILE http://localhost:8545)

# Clean up the temporary file
rm $TMP_FILE

# Check if the transaction was successful
if echo $RESPONSE | grep -q "result"; then
    TX_HASH=$(echo $RESPONSE | grep -o '"result":"0x[a-fA-F0-9]*"' | cut -d'"' -f4)
    echo -e "${GREEN}Transaction sent successfully!${NC}"
    echo -e "Transaction hash: ${YELLOW}$TX_HASH${NC}"
    echo -e "Tokens sent: ${YELLOW}$AMOUNT UBI${NC}"
    echo -e "Recipient: ${YELLOW}$TO_ADDRESS${NC}"
    
    echo
    echo -e "${GREEN}You can now check your balance in MetaMask.${NC}"
    echo -e "If you don't see the tokens immediately, try refreshing MetaMask or waiting a few seconds."
else
    ERROR=$(echo $RESPONSE | grep -o '"error":{[^}]*}' | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    echo -e "${RED}Transaction failed: $ERROR${NC}"
    echo -e "Please make sure the UBI Chain node is running and try again."
    echo -e "You can check the node logs with: tail -f node_output.log"
fi

echo
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Open MetaMask and check your balance"
echo "2. If you don't see the tokens, make sure you've added the UBI Chain network to MetaMask:"
echo "   - Network Name: UBI Chain Local"
echo "   - RPC URL: http://localhost:8545"
echo "   - Chain ID: 2030"
echo "   - Currency Symbol: UBI"
echo
echo -e "${GREEN}Happy testing!${NC}" 