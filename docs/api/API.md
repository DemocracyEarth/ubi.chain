# UBI Chain API Documentation

## Overview

UBI Chain provides two main API interfaces:
1. Native JSON-RPC API for UBI Chain-specific functionality
2. [Ethereum-compatible JSON-RPC API](ETHEREUM_COMPATIBILITY.md) for wallet integration

## RPC Endpoints

### Account Management

#### Create Account
```json
{
  "jsonrpc": "2.0",
  "method": "account_create",
  "params": ["address"],
  "id": 1
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "success": true,
    "account": {
      "address": "0x123...",
      "balance": 0,
      "verified": false
    }
  }
}
```

#### Get Account Info
```json
{
  "jsonrpc": "2.0",
  "method": "account_getInfo",
  "params": ["address"],
  "id": 1
}
```

### Chain State Queries

#### Get Account Balance
```json
{
  "jsonrpc": "2.0",
  "method": "state_getBalance",
  "params": ["address"],
  "id": 1
}
```

#### Get Verification Status
```json
{
  "jsonrpc": "2.0",
  "method": "ubi_getVerificationStatus",
  "params": ["address"],
  "id": 1
}
```

#### Get AI Resource Allocation
```json
{
  "jsonrpc": "2.0",
  "method": "ai_getResourceAllocation",
  "params": ["address"],
  "id": 1
}
```

### Transaction Submission

#### Submit Verification
```json
{
  "jsonrpc": "2.0",
  "method": "ubi_submitVerification",
  "params": [{
    "proof": "proof_data",
    "metadata": {}
  }],
  "id": 1
}
```

#### Claim UBI
```json
{
  "jsonrpc": "2.0",
  "method": "ubi_claim",
  "params": [{
    "period": "current_period"
  }],
  "id": 1
}
```

#### Request AI Resources
```json
{
  "jsonrpc": "2.0",
  "method": "ai_requestResources",
  "params": [{
    "amount": "requested_amount",
    "duration": "time_period"
  }],
  "id": 1
}
```

### Events Subscription

#### Subscribe to New Blocks
```json
{
  "jsonrpc": "2.0",
  "method": "chain_subscribeNewHeads",
  "params": [],
  "id": 1
}
```

#### Subscribe to UBI Events
```json
{
  "jsonrpc": "2.0",
  "method": "ubi_subscribeEvents",
  "params": [],
  "id": 1
}
```

## WebSocket API

### Connection
```javascript
const ws = new WebSocket('ws://localhost:9944');
```

### Event Handling
```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Handle different event types
};
```

## Ethereum Compatibility

UBI Chain provides Ethereum JSON-RPC compatibility for wallet integration. See the [Ethereum Compatibility](ETHEREUM_COMPATIBILITY.md) documentation for details.

Key features:
- Standard Ethereum JSON-RPC methods
- Chain ID: 2030
- Automatic account creation
- UBI token as native currency

## Error Codes

| Code | Description |
|------|-------------|
| 1001 | Invalid address format |
| 1002 | Verification failed |
| 1003 | Insufficient balance |
| 1004 | Rate limit exceeded |
| 1005 | Invalid proof |
| 1006 | Resource unavailable |
| 1007 | Account creation failed |

## Rate Limits

- Maximum 30 requests per minute per IP
- Maximum 5 verification attempts per day per address
- Maximum 1 UBI claim per period per address

## Security Considerations

- All endpoints require HTTPS
- Authentication via signed messages
- Rate limiting to prevent abuse
- Input validation for all parameters
- Ethereum address validation

## Example Implementation

```javascript
const UBIChain = {
  async createAccount(address) {
    return await rpcCall('account_create', [address]);
  },

  async getAccountInfo(address) {
    return await rpcCall('account_getInfo', [address]);
  },
  
  async getBalance(address) {
    return await rpcCall('state_getBalance', [address]);
  },
  
  async submitVerification(proof) {
    return await rpcCall('ubi_submitVerification', [{proof}]);
  },
  
  async claimUBI() {
    return await rpcCall('ubi_claim', [{period: getCurrentPeriod()}]);
  }
};
``` 