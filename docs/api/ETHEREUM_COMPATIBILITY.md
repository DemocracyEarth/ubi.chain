# Ethereum Compatibility

UBI Chain provides Ethereum JSON-RPC compatibility, allowing users to connect standard Ethereum wallets and tools to the network without requiring specialized clients.

## Overview

The Ethereum compatibility layer in UBI Chain implements the standard Ethereum JSON-RPC API, enabling:

- Connection from MetaMask and other Ethereum wallets
- Use of standard Ethereum development tools
- Familiar interface for Ethereum developers
- Seamless integration with existing Ethereum infrastructure

## Configuration

The Ethereum JSON-RPC server can be configured when starting a node:

```bash
cargo run --release --bin ubi-chain-node -- \
  --eth-rpc-host 127.0.0.1 \
  --eth-rpc-port 8545 \
  --chain-id 2030
```

### Options

- `--eth-rpc-host`: Host address for the Ethereum RPC server (default: 127.0.0.1)
- `--eth-rpc-port`: Port for the Ethereum RPC server (default: 8545)
- `--chain-id`: Chain ID for EIP-155 transaction signing (default: 2030)
- `--disable-eth-rpc`: Disable the Ethereum RPC server entirely

## Connecting Wallets

### MetaMask

1. Open MetaMask
2. Click on the network dropdown at the top
3. Select "Add Network" or "Custom RPC"
4. Enter the following details:
   - Network Name: UBI Chain
   - New RPC URL: http://localhost:8545 (or your custom endpoint)
   - Chain ID: 2030
   - Currency Symbol: UBI
   - Block Explorer URL: (leave blank for now)
5. Click "Save"

### Other Wallets

Most Ethereum-compatible wallets support custom networks. Use the same configuration as above, adjusting the interface as needed for your specific wallet.

## Supported Methods

The following Ethereum JSON-RPC methods are currently supported:

- `eth_chainId`: Returns the chain ID used for signing transactions
- `eth_blockNumber`: Returns the current block number
- `eth_getBalance`: Returns the balance of an account
- `eth_accounts`: Returns a list of addresses owned by the client
- `net_version`: Returns the current network ID
- `eth_gasPrice`: Returns the current gas price
- `eth_estimateGas`: Estimates gas required for a transaction
- `eth_getTransactionCount`: Returns the number of transactions sent from an address
- `eth_sendRawTransaction`: Submits a signed transaction
- `eth_getTransactionReceipt`: Returns the receipt of a transaction

## UBI Token

The native token of UBI Chain is represented as an ERC-20 compatible token with the symbol "UBI" when accessed through the Ethereum compatibility layer.

## Account Creation

When an Ethereum address is queried through the Ethereum JSON-RPC interface, an account is automatically created if it doesn't exist. This ensures seamless integration with Ethereum wallets.

## Limitations

The current implementation has the following limitations:

- No smart contract execution (EVM) support
- Limited transaction types supported
- Simplified gas model
- No support for Ethereum events/logs

## Future Enhancements

Planned enhancements to the Ethereum compatibility layer:

- Full EIP-1559 transaction support
- Enhanced transaction receipt information
- Better error reporting
- Block explorer integration
- Web3.js and ethers.js library compatibility testing 