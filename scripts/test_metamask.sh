#!/bin/bash

# Test MetaMask Integration with UBI Chain
# This script helps users test the MetaMask integration with UBI Chain

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
USE_WEB_SERVER=false
WEB_SERVER_PORT=8000

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --server) USE_WEB_SERVER=true ;;
        --port) WEB_SERVER_PORT="$2"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

echo -e "${GREEN}UBI Chain - MetaMask Integration Test${NC}"
echo "This script will help you test the MetaMask integration with UBI Chain."
echo

# Check if the node is already running
if pgrep -f "ubi-chain-node" > /dev/null; then
    echo -e "${YELLOW}UBI Chain node is already running.${NC}"
    echo "If you want to restart it, please stop the current instance first."
    echo
else
    echo -e "${YELLOW}Starting UBI Chain node with Ethereum RPC support...${NC}"
    echo "The node will run in the background. To stop it, use 'pkill -f ubi-chain-node'"
    
    # Start the node in the background
    cargo run --bin ubi-chain-node -- --eth-rpc-host 127.0.0.1 --eth-rpc-port 8545 --chain-id 2030 > node_output.log 2>&1 &
    
    # Wait for the node to start
    sleep 5
    
    if pgrep -f "ubi-chain-node" > /dev/null; then
        echo -e "${GREEN}Node started successfully!${NC}"
    else
        echo -e "${RED}Failed to start the node. Check node_output.log for details.${NC}"
        exit 1
    fi
fi

echo
echo -e "${YELLOW}MetaMask Configuration:${NC}"
echo "1. Open MetaMask and add a new network with these details:"
echo "   - Network Name: UBI Chain Local"
echo "   - RPC URL: http://localhost:8545"
echo "   - Chain ID: 2030"
echo "   - Currency Symbol: UBI"
echo

# If using web server
if [ "$USE_WEB_SERVER" = true ]; then
    echo -e "${YELLOW}Starting local web server on port $WEB_SERVER_PORT...${NC}"
    
    # Check if Python is installed
    if command -v python3 &>/dev/null; then
        PYTHON_CMD="python3"
    elif command -v python &>/dev/null; then
        PYTHON_CMD="python"
    else
        echo -e "${RED}Python is not installed. Cannot start web server.${NC}"
        echo "Please install Python or run without the --server option."
        exit 1
    fi
    
    # Start a local web server in the background
    echo "Starting web server with: $PYTHON_CMD -m http.server $WEB_SERVER_PORT"
    $PYTHON_CMD -m http.server $WEB_SERVER_PORT > web_server.log 2>&1 &
    WEB_SERVER_PID=$!
    
    # Wait for the server to start
    sleep 2
    
    if kill -0 $WEB_SERVER_PID 2>/dev/null; then
        echo -e "${GREEN}Web server started successfully!${NC}"
        echo "To stop the web server, run: kill $WEB_SERVER_PID"
        
        # Open the web page in the browser
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            xdg-open "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_integration.html"
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            open "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_integration.html"
        elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
            start "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_integration.html"
        else
            echo -e "${RED}Unsupported OS. Please open the URL manually:${NC}"
            echo "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_integration.html"
        fi
    else
        echo -e "${RED}Failed to start web server. Check web_server.log for details.${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}Opening MetaMask Integration Page:${NC}"
    echo "Opening the MetaMask integration page in your default browser..."
    
    # Determine the OS and open the browser accordingly
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        xdg-open "file://$(pwd)/docs/tutorials/metamask_integration.html"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        open "file://$(pwd)/docs/tutorials/metamask_integration.html"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        start "file://$(pwd)/docs/tutorials/metamask_integration.html"
    else
        echo -e "${RED}Unsupported OS. Please open the file manually:${NC}"
        echo "$(pwd)/docs/tutorials/metamask_integration.html"
    fi
fi

echo
echo -e "${GREEN}Test Instructions:${NC}"
echo "1. Connect your MetaMask wallet using the 'Connect MetaMask' button"
echo "2. Send a test transaction to another address"
echo "3. Check the transaction status in MetaMask"
echo
echo -e "${YELLOW}Advanced Web3.js Example:${NC}"
echo "For more advanced interactions using Web3.js, you can open:"
if [ "$USE_WEB_SERVER" = true ]; then
    echo "http://localhost:$WEB_SERVER_PORT/docs/tutorials/web3_example.html"
else
    echo "$(pwd)/docs/tutorials/web3_example.html"
fi
echo
echo -e "${YELLOW}Node Logs:${NC}"
echo "To view the node logs, run: tail -f node_output.log"
echo
echo -e "${YELLOW}Troubleshooting:${NC}"
echo "If you see 'ethereum is not defined' errors, try running with the web server option:"
echo "./scripts/test_metamask.sh --server"
echo
echo -e "${GREEN}Happy testing!${NC}" 