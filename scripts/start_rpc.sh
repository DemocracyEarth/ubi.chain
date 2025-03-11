#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
HTTP_HOST="127.0.0.1"
WS_HOST="127.0.0.1"
HTTP_PORT=8545
WS_PORT=8546
CHAIN_ID=2030
LOG_LEVEL="info"
BINARY_NAME="eth_rpc_server"

# Print banner
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}   UBI Chain RPC Server Start   ${NC}"
echo -e "${GREEN}================================${NC}\n"

# Function to cleanup existing processes
cleanup() {
    echo -e "${YELLOW}Cleaning up existing processes...${NC}"
    
    # Kill any existing eth_rpc_server processes
    if pgrep -f "$BINARY_NAME" > /dev/null; then
        echo -e "${YELLOW}Found existing RPC server processes. Stopping them...${NC}"
        pkill -f "$BINARY_NAME"
        sleep 2
    fi
    
    # Force kill if any processes are still running
    if pgrep -f "$BINARY_NAME" > /dev/null; then
        echo -e "${YELLOW}Force stopping remaining processes...${NC}"
        pkill -9 -f "$BINARY_NAME"
        sleep 1
    fi
    
    # Kill any processes using our ports
    for PORT in $HTTP_PORT $WS_PORT; do
        if lsof -ti:$PORT > /dev/null; then
            echo -e "${YELLOW}Killing process using port $PORT...${NC}"
            lsof -ti:$PORT | xargs kill -9
            sleep 1
        fi
    done
    
    echo -e "${GREEN}Cleanup completed${NC}"
}

# Function to check if a port is in use
check_port() {
    if lsof -i:$1 > /dev/null; then
        echo -e "${RED}Error: Port $1 is still in use after cleanup${NC}"
        return 1
    fi
    return 0
}

# Function to check if the server is responding
check_server() {
    local port=$1
    local max_attempts=5
    local attempt=1
    
    echo -e "${YELLOW}Checking server health on port $port...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            http://$HTTP_HOST:$port > /dev/null; then
            echo -e "${GREEN}Server is responding on port $port${NC}"
            return 0
        fi
        
        echo -e "${YELLOW}Attempt $attempt of $max_attempts...${NC}"
        sleep 2
        ((attempt++))
    done
    
    echo -e "${RED}Server failed to respond on port $port${NC}"
    return 1
}

# Run cleanup first
cleanup

# Check if ports are available
check_port $HTTP_PORT || exit 1
check_port $WS_PORT || exit 1

# Create log directory if it doesn't exist
mkdir -p logs

# Clear previous log file
echo "" > logs/rpc_server.log

# Start the server with logging
echo -e "${GREEN}Starting RPC server...${NC}"
echo -e "HTTP endpoint: http://$HTTP_HOST:$HTTP_PORT"
echo -e "WebSocket endpoint: ws://$WS_HOST:$WS_PORT"
echo -e "Chain ID: $CHAIN_ID\n"

cd "$(dirname "$0")/.." && RUST_LOG=$LOG_LEVEL cargo run --bin eth_rpc_server \
    "$HTTP_HOST:$HTTP_PORT" \
    "$WS_HOST:$WS_PORT" \
    $CHAIN_ID > logs/rpc_server.log 2>&1 &

SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait a moment before checking
sleep 2

# Check if process is still running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}Server failed to start. Check logs/rpc_server.log for details${NC}"
    echo -e "${YELLOW}Last few lines of the log:${NC}"
    tail -n 10 logs/rpc_server.log
    exit 1
fi

# Check server health
check_server $HTTP_PORT

# Print helpful commands
echo -e "\n${GREEN}Helpful commands:${NC}"
echo -e "- Check server status: ${YELLOW}curl -X POST -H \"Content-Type: application/json\" --data '{\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\",\"params\":[],\"id\":1}' http://$HTTP_HOST:$HTTP_PORT${NC}"
echo -e "- View logs: ${YELLOW}tail -f logs/rpc_server.log${NC}"
echo -e "- Stop server: ${YELLOW}kill $SERVER_PID${NC}"
echo -e "- Stop all servers: ${YELLOW}pkill -f $BINARY_NAME${NC}"

echo -e "\n${GREEN}Server is running in the background. Use the commands above to manage it.${NC}" 