#!/bin/bash

# Direct token transfer script for UBI Chain
# This script sends tokens directly to a MetaMask address without requiring the web interface

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
AMOUNT=100
FROM_ADDRESS="0x1111111111111111111111111111111111111111"

echo -e "${GREEN}UBI Chain - Direct Token Transfer${NC}"
echo "This script sends tokens directly to your MetaMask address."
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

# Create a JSON-RPC request
JSON_REQUEST=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
        "from": "$FROM_ADDRESS",
        "to": "$TO_ADDRESS",
        "value": "0x$(printf '%064x' $AMOUNT)0000000000000000",
        "gas": "0x5208",
        "gasPrice": "0x3b9aca00"
    }],
    "id": 1
}
EOF
)

# Send the request to the RPC endpoint
echo -e "${YELLOW}Sending request to RPC endpoint...${NC}"
RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$JSON_REQUEST" http://localhost:8545)

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
    ERROR=$(echo $RESPONSE | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    echo -e "${RED}Transaction failed: $ERROR${NC}"
    echo -e "Please make sure the UBI Chain node is running and try again."
    echo -e "You can start the node with: ./scripts/test_metamask.sh --server"
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