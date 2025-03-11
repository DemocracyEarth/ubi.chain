# Ethereum-compatible JSON-RPC Server for UBI Chain

This module implements an Ethereum-compatible JSON-RPC server for UBI Chain, supporting both HTTP pull requests and WebSocket push notifications. It allows Ethereum wallets like MetaMask to connect to the UBI Chain without implementing the Ethereum Virtual Machine.

## Features

- **HTTP JSON-RPC Server**: Implements standard Ethereum JSON-RPC methods over HTTP
- **WebSocket Server**: Provides real-time notifications for new blocks and transactions
- **Subscription Support**: Implements `eth_subscribe` and `eth_unsubscribe` methods for WebSocket clients
- **Ethereum Compatibility**: Compatible with Ethereum wallets and tools

## Supported Methods

### HTTP and WebSocket Methods

- `eth_getBalance`: Get account balance in wei
- `eth_sendTransaction`: Send a transaction
- `eth_getTransactionCount`: Get the number of transactions sent from an address
- `eth_chainId`: Get the chain ID
- `eth_blockNumber`: Get the current block number
- `eth_getBlockByNumber`: Get block information by block number
- `eth_getBlockByHash`: Get block information by block hash
- `eth_accounts`: Get a list of addresses owned by the client
- `eth_sendRawTransaction`: Send a signed transaction
- `eth_getTransactionReceipt`: Get transaction receipt
- `eth_getTransactionByHash`: Get transaction information by hash
- `eth_estimateGas`: Estimate gas for a transaction
- `eth_getLogs`: Get logs matching a filter

### WebSocket-specific Methods

- `eth_subscribe`: Subscribe to events (newHeads, newPendingTransactions, logs)
- `eth_unsubscribe`: Unsubscribe from events

## Usage

### Starting the Server

```rust
use ubi_chain_rpc::RpcHandler;
use runtime::Runtime;

// Initialize the runtime
let runtime = Runtime::new();

// Create the RPC handler
let rpc_handler = RpcHandler::new(runtime);

// Start both HTTP and WebSocket servers
let servers = rpc_handler.start_eth_rpc_servers("127.0.0.1:8545", "127.0.0.1:8546", 2030)
    .expect("Failed to start servers");

// Keep the servers running
// ...
```

### Using the Example Binary

The module includes an example binary that demonstrates how to use both the HTTP and WebSocket servers:

```bash
# Build the example binary
cargo build --bin eth_rpc_server

# Run the server with default settings (HTTP: 127.0.0.1:8545, WS: 127.0.0.1:8546, Chain ID: 2030)
./target/debug/eth_rpc_server

# Run the server with custom settings
./target/debug/eth_rpc_server 0.0.0.0:8545 0.0.0.0:8546 1337
```

### Testing with the Provided Scripts

#### Shell Script

The `scripts/test_eth_rpc.sh` script demonstrates how to interact with the server using curl:

```bash
# Make the script executable
chmod +x scripts/test_eth_rpc.sh

# Run the test script
./scripts/test_eth_rpc.sh
```

#### Web Interface

The `scripts/websocket_test.html` file provides a simple web interface for testing WebSocket subscriptions:

1. Start the server using the example binary
2. Open the HTML file in a web browser
3. Connect to the WebSocket server
4. Subscribe to events
5. Send transactions to see real-time notifications

### Connecting with MetaMask

1. Open MetaMask and click on the network dropdown
2. Select "Add Network"
3. Enter the following details:
   - Network Name: UBI Chain
   - RPC URL: http://localhost:8545
   - Chain ID: 2030
   - Currency Symbol: UBI
4. Click "Save"
5. You should now be connected to the UBI Chain

## Implementation Details

### Architecture

The server implementation consists of the following components:

- `EthRpcHandler`: Handles HTTP JSON-RPC requests
- `EthPubSubHandler`: Handles WebSocket subscriptions
- `SubscriptionManager`: Manages WebSocket subscriptions and notifications

### Subscription Flow

1. Client connects to the WebSocket server
2. Client sends an `eth_subscribe` request with the subscription type
3. Server registers the subscription and returns a subscription ID
4. When an event occurs (new block, new transaction), the server notifies all relevant subscribers
5. Client can unsubscribe using the `eth_unsubscribe` method

## Development

### Adding New Methods

To add a new method to the server:

1. Implement the method in the `EthRpcHandler` struct
2. Add the method to the IoHandler in the `start_server` method
3. Add the method to the WebSocket handler in the `start_eth_ws_server` method

### Adding New Subscription Types

To add a new subscription type:

1. Add the type to the `SubscriptionType` enum in `eth_pubsub.rs`
2. Update the `FromStr` implementation for the new type
3. Implement a notification method in the `SubscriptionManager` struct
4. Call the notification method when the relevant event occurs

## License

This module is part of the UBI Chain project and is licensed under the same terms as the main project. 