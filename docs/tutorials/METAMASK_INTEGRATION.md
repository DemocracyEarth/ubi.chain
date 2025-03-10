# Using MetaMask with UBI Chain

This tutorial explains how to connect MetaMask to your local UBI Chain node and perform transactions.

## Prerequisites

1. [MetaMask](https://metamask.io/) browser extension installed
2. UBI Chain node running locally

## Setting Up MetaMask

### 1. Add UBI Chain Network to MetaMask

1. Open MetaMask and click on the network dropdown at the top
2. Select "Add Network" > "Add a network manually"
3. Fill in the following details:
   - **Network Name**: UBI Chain Local
   - **New RPC URL**: http://localhost:8545
   - **Chain ID**: 2030
   - **Currency Symbol**: UBI
   - **Block Explorer URL**: (leave blank)
4. Click "Save"

### 2. Import Test Accounts (Optional)

If you want to use the test accounts that come with UBI Chain:

1. In MetaMask, click on your account icon > "Import Account"
2. Enter one of the following private keys:
   - `0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d` (Test Account 1)
   - `0x6cbed15c793ce57650b9877cf6fa156fbef513c4e6134f022a85b1ffdd59b2a1` (Test Account 2)
   - `0x6370fd033278c143179d81c5526140625662b8daa446c22ee2d73db3707e620c` (Test Account 3)
3. Click "Import"

## Using the Web Interface

We've created a simple web interface to interact with UBI Chain using MetaMask.

1. Open the file `docs/tutorials/metamask_integration.html` in your browser
2. Click "Connect MetaMask" to connect your MetaMask wallet
3. Once connected, you can:
   - View your account information
   - Send transactions to other addresses
   - Get chain information

## Sending Transactions Programmatically

If you want to send transactions programmatically, you can use the following JavaScript code:

```javascript
// Connect to MetaMask
const accounts = await ethereum.request({ method: 'eth_requestAccounts' });
const account = accounts[0];

// Send transaction
const txHash = await ethereum.request({
  method: 'eth_sendTransaction',
  params: [{
    from: account,
    to: '0xRecipientAddress',
    value: '0xValueInWei', // Hex string of the value in wei
    gas: '0x5208', // 21000 gas
    gasPrice: '0x3b9aca00', // 1 Gwei
  }],
});

console.log('Transaction hash:', txHash);
```

## Troubleshooting

### Transaction Fails with "Invalid Sender"

This usually means that the account you're using in MetaMask doesn't exist on the UBI Chain. Try sending a small amount to your MetaMask account from one of the test accounts first.

### MetaMask Can't Connect to Network

Make sure your UBI Chain node is running and the Ethereum RPC server is enabled. You can start the node with:

```bash
cargo run --bin ubi-chain-node -- --eth-rpc-host 127.0.0.1 --eth-rpc-port 8545
```

### Wrong Chain ID

If you see errors about the wrong chain ID, make sure you've configured MetaMask with chain ID 2030, which is the default for UBI Chain.

### "ethereum is not defined" Error

If you see an error like "Error connecting to MetaMask: ethereum is not defined", this means the web page cannot detect the MetaMask extension. Try the following:

1. **Make sure MetaMask is installed**: Install MetaMask from [metamask.io](https://metamask.io/download.html) if you haven't already.

2. **Refresh the page**: Sometimes a simple page refresh can resolve the issue.

3. **Unlock MetaMask**: Make sure your MetaMask wallet is unlocked.

4. **Check browser permissions**: Make sure MetaMask has permission to access the page. Click on the MetaMask icon in your browser and check the connection status.

5. **Try a different browser**: If you're using a browser that's not fully supported by MetaMask, try using Chrome or Firefox.

6. **Open the file correctly**: When opening the HTML files, make sure you're using the `file://` protocol. The test script should handle this automatically, but if you're opening the files manually, use:
   ```
   file:///path/to/ubi-chain/docs/tutorials/metamask_integration.html
   ```

7. **Use a local web server**: For better compatibility, you can serve the files using a local web server:
   ```bash
   # Using Python's built-in HTTP server
   cd /path/to/ubi-chain
   python -m http.server 8000
   ```
   Then access the page at `http://localhost:8000/docs/tutorials/metamask_integration.html`

## Advanced: Using Web3.js

For more advanced interactions, you can use Web3.js:

```javascript
// Connect to Web3
const web3 = new Web3(window.ethereum);

// Get accounts
const accounts = await web3.eth.getAccounts();

// Get balance
const balance = await web3.eth.getBalance(accounts[0]);
console.log('Balance:', web3.utils.fromWei(balance, 'ether'), 'UBI');

// Send transaction
const tx = await web3.eth.sendTransaction({
  from: accounts[0],
  to: '0xRecipientAddress',
  value: web3.utils.toWei('1', 'ether'),
});
console.log('Transaction:', tx);
```

We've also created a complete Web3.js example that you can use as a reference:
1. Open the file `docs/tutorials/web3_example.html` in your browser
2. This example demonstrates how to:
   - Connect to Web3 using MetaMask
   - Get account information
   - Get chain information
   - Send transactions
   - Handle events like account changes

## Next Steps

- Learn how to [verify your account](./VERIFICATION.md) to receive UBI
- Explore the [UBI distribution mechanism](../architecture/UBI_DISTRIBUTION.md)
- Contribute to the [UBI Chain development](../contributing/CONTRIBUTING.md) 