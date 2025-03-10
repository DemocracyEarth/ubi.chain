//! Ethereum JSON-RPC Compatibility Module
//!
//! This module implements Ethereum JSON-RPC compatibility for UBI Chain,
//! allowing Ethereum wallets to connect to the node without implementing
//! the Ethereum Virtual Machine.

use crate::RpcHandler;
use jsonrpc_core::{Error, Result, Value};
use jsonrpc_core::futures::future;
use jsonrpc_http_server::{Server, ServerBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use hex;
use rand;

// Helper macro for cloning handlers
macro_rules! clone_handler {
    ($handler:expr, $method:ident) => {
        {
            let handler = $handler.clone();
            move |params| {
                let handler = handler.clone();
                async move { handler.$method(params).await }
            }
        }
    };
}

/// Validates if a string is a valid Ethereum address
///
/// # Arguments
/// * `address` - The address string to validate
///
/// # Returns
/// true if the address is valid, false otherwise
fn is_valid_eth_address(address: &str) -> bool {
    // Ethereum addresses are 0x followed by 40 hex characters
    if !address.starts_with("0x") || address.len() != 42 {
        return false;
    }
    
    // Check if all characters after 0x are valid hex
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Ethereum-compatible block information
#[derive(Debug, Serialize, Deserialize)]
pub struct EthBlock {
    pub number: String,
    pub hash: String,
    pub parent_hash: String,
    pub nonce: String,
    pub sha3_uncles: String,
    pub logs_bloom: String,
    pub transactions_root: String,
    pub state_root: String,
    pub receipts_root: String,
    pub miner: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub extra_data: String,
    pub size: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: String,
    pub transactions: Vec<Value>,
    pub uncles: Vec<String>,
}

/// Ethereum-compatible transaction information
#[derive(Debug, Serialize, Deserialize)]
pub struct EthTransaction {
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub block_number: String,
    pub transaction_index: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_price: String,
    pub gas: String,
    pub input: String,
    pub v: String,
    pub r: String,
    pub s: String,
}

/// Ethereum-compatible account information
#[derive(Debug, Serialize, Deserialize)]
pub struct EthAccount {
    pub balance: String,
    pub code: String,
    pub nonce: String,
    pub storage_hash: String,
}

/// Ethereum JSON-RPC handler
pub struct EthRpcHandler {
    /// Reference to the UBI Chain RPC handler
    rpc_handler: RpcHandler,
    /// Chain ID for EIP-155 compatibility
    chain_id: u64,
}

impl EthRpcHandler {
    /// Creates a new Ethereum-compatible RPC handler
    ///
    /// # Arguments
    /// * `rpc_handler` - The UBI Chain RPC handler
    /// * `chain_id` - Chain ID for EIP-155 compatibility
    pub fn new(rpc_handler: RpcHandler, chain_id: u64) -> Self {
        EthRpcHandler {
            rpc_handler,
            chain_id,
        }
    }
    
    /// Starts the JSON-RPC server
    ///
    /// # Arguments
    /// * `addr` - The address to bind the server to
    ///
    /// # Returns
    /// The server instance
    ///
    /// # Important
    /// The returned server instance must be stored in a variable that lives for the duration
    /// of the program to prevent the Tokio runtime from being dropped in an asynchronous context.
    /// Dropping the server in an asynchronous context will cause a panic with:
    /// "Cannot drop a runtime in a context where blocking is not allowed."
    pub fn start_server(self, addr: &str) -> Result<Server> {
        let addr = SocketAddr::from_str(addr).map_err(|_| {
            Error::invalid_params("Invalid address format")
        })?;
        
        let handler = Arc::new(self);
        let mut io = jsonrpc_core::IoHandler::new();
        
        // Register Ethereum-compatible methods
        io.add_method("eth_chainId", clone_handler!(handler, eth_chain_id));
        io.add_method("eth_blockNumber", clone_handler!(handler, eth_block_number));
        io.add_method("eth_getBalance", clone_handler!(handler, eth_get_balance));
        io.add_method("eth_sendTransaction", clone_handler!(handler, eth_send_transaction));
        io.add_method("eth_getTransactionCount", clone_handler!(handler, eth_get_transaction_count));
        io.add_method("eth_getBlockByNumber", clone_handler!(handler, eth_get_block_by_number));
        io.add_method("eth_getBlockByHash", clone_handler!(handler, eth_get_block_by_hash));
        io.add_method("eth_accounts", clone_handler!(handler, eth_accounts));
        
        // Start the server with a proper configuration to avoid runtime issues
        let server = ServerBuilder::new(io)
            .threads(4)
            .start_http(&addr)
            .map_err(|_e| Error::invalid_request())?;
            
        // Return the server without dropping it
        Ok(server)
    }
    
    /// Implements eth_getBalance
    ///
    /// Gets the balance of an account at a given block
    ///
    /// # Parameters
    /// * `params` - [address, block_identifier]
    ///
    /// # Returns
    /// The balance in wei (converted from UBI tokens)
    pub fn eth_get_balance(&self, params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let params = match params.parse::<Vec<Value>>() {
            Ok(p) => p,
            Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid parameters")))),
        };
        
        if params.is_empty() {
            return Box::pin(future::ready(Err(Error::invalid_params("Missing address parameter"))));
        }
        
        let address = match params[0].as_str() {
            Some(addr) => addr,
            None => return Box::pin(future::ready(Err(Error::invalid_params("Invalid address format")))),
        };
        
        if !is_valid_eth_address(address) {
            return Box::pin(future::ready(Err(Error::invalid_params("Invalid Ethereum address"))));
        }
        
        // Get balance from UBI Chain runtime
        let balance = self.rpc_handler.get_account_info(address.to_string()).balance;
        
        // Convert to wei (1 UBI token = 10^18 wei for Ethereum compatibility)
        let wei_balance = balance.saturating_mul(1_000_000_000_000_000_000);
        
        // Format as hex string with 0x prefix
        let hex_balance = format!("0x{:x}", wei_balance);
        
        Box::pin(future::ready(Ok(Value::String(hex_balance))))
    }
    
    /// Implements eth_sendTransaction
    ///
    /// Sends a transaction to the UBI Chain
    ///
    /// # Parameters
    /// * `params` - [{from, to, value, ...}]
    ///
    /// # Returns
    /// The transaction hash
    pub fn eth_send_transaction(&self, params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let params = match params.parse::<Vec<Value>>() {
            Ok(p) => p,
            Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid parameters")))),
        };
        
        if params.is_empty() {
            return Box::pin(future::ready(Err(Error::invalid_params("Missing transaction parameter"))));
        }
        
        let tx_obj = match params[0].as_object() {
            Some(obj) => obj,
            None => return Box::pin(future::ready(Err(Error::invalid_params("Transaction must be an object")))),
        };
        
        // Extract transaction parameters
        let from = match tx_obj.get("from").and_then(|v| v.as_str()) {
            Some(addr) => addr,
            None => return Box::pin(future::ready(Err(Error::invalid_params("Missing 'from' address")))),
        };
        
        let to = match tx_obj.get("to").and_then(|v| v.as_str()) {
            Some(addr) => addr,
            None => return Box::pin(future::ready(Err(Error::invalid_params("Missing 'to' address")))),
        };
        
        // Validate addresses
        if !is_valid_eth_address(from) || !is_valid_eth_address(to) {
            return Box::pin(future::ready(Err(Error::invalid_params("Invalid Ethereum address"))));
        }
        
        // Parse value (in wei)
        let value_wei = match tx_obj.get("value") {
            Some(val) => {
                match val.as_str() {
                    Some(hex_val) => {
                        if let Some(stripped) = hex_val.strip_prefix("0x") {
                            match u64::from_str_radix(stripped, 16) {
                                Ok(v) => v,
                                Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid value format")))),
                            }
                        } else {
                            match hex_val.parse::<u64>() {
                                Ok(v) => v,
                                Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid value format")))),
                            }
                        }
                    },
                    None => match val.as_u64() {
                        Some(v) => v,
                        None => return Box::pin(future::ready(Err(Error::invalid_params("Invalid value format")))),
                    }
                }
            },
            None => 0, // Default to 0 if not specified
        };
        
        // Convert from wei to UBI tokens (1 UBI token = 10^18 wei)
        let value_ubi = value_wei / 1_000_000_000_000_000_000;
        
        // Execute the transfer
        match self.rpc_handler.runtime.transfer_with_fee(from, to, value_ubi) {
            Ok(_) => {
                // Generate a transaction hash
                let mut tx_hash = [0u8; 32];
                rand::Rng::fill(&mut rand::thread_rng(), &mut tx_hash);
                let tx_hash_hex = format!("0x{}", hex::encode(tx_hash));
                
                Box::pin(future::ready(Ok(Value::String(tx_hash_hex))))
            },
            Err(e) => {
                let error_msg = match e {
                    runtime::AccountError::AlreadyExists => "Account already exists",
                    runtime::AccountError::InvalidAddress => "Invalid address",
                    runtime::AccountError::Other(ref msg) => msg.as_str(),
                };
                
                Box::pin(future::ready(Err(Error::invalid_params(error_msg))))
            }
        }
    }
    
    /// Implements eth_getTransactionCount
    ///
    /// Gets the number of transactions sent from an address
    /// (In UBI Chain, we don't track nonces, so this is a placeholder)
    ///
    /// # Parameters
    /// * `params` - [address, block_identifier]
    ///
    /// # Returns
    /// The transaction count as a hex string
    pub fn eth_get_transaction_count(&self, params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let params = match params.parse::<Vec<Value>>() {
            Ok(p) => p,
            Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid parameters")))),
        };
        
        if params.is_empty() {
            return Box::pin(future::ready(Err(Error::invalid_params("Missing address parameter"))));
        }
        
        let address = match params[0].as_str() {
            Some(addr) => addr,
            None => return Box::pin(future::ready(Err(Error::invalid_params("Invalid address format")))),
        };
        
        if !is_valid_eth_address(address) {
            return Box::pin(future::ready(Err(Error::invalid_params("Invalid Ethereum address"))));
        }
        
        // In UBI Chain, we don't track nonces, so return 0
        Box::pin(future::ready(Ok(Value::String("0x0".to_string()))))
    }
    
    /// Implements eth_chainId
    ///
    /// Returns the chain ID used for signing replay-protected transactions
    ///
    /// # Returns
    /// The chain ID as a hex string
    pub fn eth_chain_id(&self, _params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let chain_id = format!("0x{:x}", self.chain_id);
        Box::pin(future::ready(Ok(Value::String(chain_id))))
    }
    
    /// Implements eth_blockNumber
    ///
    /// Returns the current block number
    ///
    /// # Returns
    /// The current block number as a hex string
    pub fn eth_block_number(&self, _params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        Box::pin(future::ready(Ok(Value::String("0x1".to_string()))))
    }
    
    /// Implements eth_getBlockByNumber
    ///
    /// Returns information about a block by block number
    ///
    /// # Parameters
    /// * `params` - [block_number, include_transactions]
    ///
    /// # Returns
    /// Block information
    pub fn eth_get_block_by_number(&self, params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let params = match params.parse::<Vec<Value>>() {
            Ok(p) => p,
            Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid parameters")))),
        };
        
        if params.is_empty() {
            return Box::pin(future::ready(Err(Error::invalid_params("Missing block number parameter"))));
        }
        
        // Create a mock block response
        let block = json!({
            "number": "0x1",
            "hash": format!("0x{}", hex::encode([1u8; 32])),
            "parentHash": format!("0x{}", hex::encode([0u8; 32])),
            "nonce": "0x0000000000000000",
            "sha3Uncles": format!("0x{}", hex::encode([0u8; 32])),
            "logsBloom": format!("0x{}", hex::encode([0u8; 32])),
            "transactionsRoot": format!("0x{}", hex::encode([0u8; 32])),
            "stateRoot": format!("0x{}", hex::encode([0u8; 32])),
            "receiptsRoot": format!("0x{}", hex::encode([0u8; 32])),
            "miner": "0x0000000000000000000000000000000000000000",
            "difficulty": "0x0",
            "totalDifficulty": "0x0",
            "extraData": "0x",
            "size": "0x1000",
            "gasLimit": "0x1000000",
            "gasUsed": "0x0",
            "timestamp": "0x5f5e100",
            "transactions": [],
            "uncles": []
        });
        
        Box::pin(future::ready(Ok(block)))
    }
    
    /// Implements eth_getBlockByHash
    ///
    /// Returns information about a block by hash
    ///
    /// # Parameters
    /// * `params` - [block_hash, include_transactions]
    ///
    /// # Returns
    /// Block information
    pub fn eth_get_block_by_hash(&self, params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let params = match params.parse::<Vec<Value>>() {
            Ok(p) => p,
            Err(_) => return Box::pin(future::ready(Err(Error::invalid_params("Invalid parameters")))),
        };
        
        if params.is_empty() {
            return Box::pin(future::ready(Err(Error::invalid_params("Missing block hash parameter"))));
        }
        
        // Create a mock block response (same as eth_getBlockByNumber)
        let block = json!({
            "number": "0x1",
            "hash": format!("0x{}", hex::encode([1u8; 32])),
            "parentHash": format!("0x{}", hex::encode([0u8; 32])),
            "nonce": "0x0000000000000000",
            "sha3Uncles": format!("0x{}", hex::encode([0u8; 32])),
            "logsBloom": format!("0x{}", hex::encode([0u8; 32])),
            "transactionsRoot": format!("0x{}", hex::encode([0u8; 32])),
            "stateRoot": format!("0x{}", hex::encode([0u8; 32])),
            "receiptsRoot": format!("0x{}", hex::encode([0u8; 32])),
            "miner": "0x0000000000000000000000000000000000000000",
            "difficulty": "0x0",
            "totalDifficulty": "0x0",
            "extraData": "0x",
            "size": "0x1000",
            "gasLimit": "0x1000000",
            "gasUsed": "0x0",
            "timestamp": "0x5f5e100",
            "transactions": [],
            "uncles": []
        });
        
        Box::pin(future::ready(Ok(block)))
    }
    
    /// Implements eth_accounts
    ///
    /// Returns a list of addresses owned by client
    ///
    /// # Returns
    /// Array of addresses
    pub fn eth_accounts(&self, _params: jsonrpc_core::Params) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Value>> {
        let mock_address = "0x0000000000000000000000000000000000000001";
        Box::pin(future::ready(Ok(Value::Array(vec![Value::String(mock_address.to_string())]))))
    }
} 