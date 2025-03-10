#!/bin/bash

# Test MetaMask with a direct connection to the Ethereum mainnet
# This script helps users test if MetaMask is working properly

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
WEB_SERVER_PORT=8000

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --port) WEB_SERVER_PORT="$2"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

echo -e "${GREEN}MetaMask Direct Connection Test${NC}"
echo "This script will help you test if MetaMask is working properly with a direct connection to the Ethereum mainnet."
echo

echo -e "${YELLOW}Starting local web server on port $WEB_SERVER_PORT...${NC}"

# Check if Python is installed
if command -v python3 &>/dev/null; then
    PYTHON_CMD="python3"
elif command -v python &>/dev/null; then
    PYTHON_CMD="python"
else
    echo -e "${RED}Python is not installed. Cannot start web server.${NC}"
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
    
    # Open the test page in the browser
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        xdg-open "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_test.html"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        open "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_test.html"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        start "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_test.html"
    else
        echo -e "${RED}Unsupported OS. Please open the URL manually:${NC}"
        echo "http://localhost:$WEB_SERVER_PORT/docs/tutorials/metamask_test.html"
    fi
else
    echo -e "${RED}Failed to start web server. Check web_server.log for details.${NC}"
    exit 1
fi

echo
echo -e "${GREEN}Test Instructions:${NC}"
echo "1. The test page will check if MetaMask is detected"
echo "2. Click 'Connect to MetaMask' to test the connection"
echo "3. Check the debug information for details about your MetaMask setup"
echo
echo -e "${YELLOW}Troubleshooting:${NC}"
echo "If MetaMask is not detected:"
echo "1. Make sure MetaMask is installed and unlocked"
echo "2. Try refreshing the page"
echo "3. Check if MetaMask is enabled for the site"
echo "4. Try using Chrome or Firefox if you're using a different browser"
echo
echo -e "${GREEN}Happy testing!${NC}" 