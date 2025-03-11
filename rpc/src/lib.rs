//! UBI Chain RPC Implementation
//!
//! This module provides the JSON-RPC interface for external interaction with the blockchain.
//! It implements:
//! - Account information queries
//! - Transaction submission
//! - UBI claim processing
//! - Verification status checks
//! - AI resource management
//! - Network status information

use runtime::{Runtime, AccountError, Transaction};
use serde::{Deserialize, Serialize};
use log::{info, warn, error};

// Add Ethereum compatibility module
pub mod eth_compat;
// Add Ethereum PubSub module
pub mod eth_pubsub;

// Remove the external crate reference
// extern crate ubi_chain_node as node;
// use node::Transaction;

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::fmt;
use jsonrpc_core::{IoHandler, Error as JsonRpcError};
use jsonrpc_http_server::Server as HttpServer;
use jsonrpc_ws_server::{Server as WsServer, ServerBuilder as WsServerBuilder};
use rand::Rng;
use hex;

/// Account information structure returned by RPC queries
///
/// This structure represents the publicly accessible information
/// about an account on the UBI Chain.
///
/// # Fields
/// * `address` - The account's unique identifier
/// * `balance` - Current UBI token balance
/// * `verified` - Human verification status
///
/// # Example Response
/// ```json
/// {
///     "address": "0x123...",
///     "balance": 1000,
///     "verified": true
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    /// The account's unique address
    address: String,
    
    /// Current balance in UBI tokens
    balance: u64,
    
    /// Whether the account has passed human verification
    verified: bool,
}

/// Response for account creation
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountResponse {
    /// Success status
    success: bool,
    
    /// Account information if successful
    account: Option<AccountInfo>,
    
    /// Error message if unsuccessful
    error: Option<String>,
}

/// Response for faucet requests
#[derive(Debug, Serialize, Deserialize)]
pub struct FaucetResponse {
    /// Success status
    pub success: bool,
    
    /// Amount of tokens sent
    pub amount: Option<u64>,
    
    /// New balance after faucet distribution
    pub new_balance: Option<u64>,
    
    /// Transaction hash (if a transaction was created)
    pub transaction_hash: Option<String>,
    
    /// Error message if unsuccessful
    pub error: Option<String>,
}

/// RPC handler for UBI Chain
///
/// This struct provides methods for handling RPC requests
/// to the UBI Chain node.
#[derive(Clone)]
pub struct RpcHandler {
    /// Reference to the blockchain runtime
    pub runtime: Runtime,
    
    /// Node address (used as the faucet address)
    pub node_address: Option<String>,
}

/// Combined server structure holding both HTTP and WebSocket servers
pub struct CombinedServer {
    /// HTTP server instance
    pub http_server: HttpServer,
    /// WebSocket server instance
    pub ws_server: WsServer,
}

impl RpcHandler {
    /// Creates a new RPC handler
    ///
    /// # Arguments
    /// * `runtime` - The blockchain runtime
    ///
    /// # Returns
    /// A new RPC handler instance
    pub fn new(runtime: Runtime) -> Self {
        RpcHandler {
            runtime,
            node_address: None,
        }
    }
    
    /// Sets the node address
    pub fn set_node_address(&mut self, address: String) {
        self.node_address = Some(address);
    }
    
    /// Gets the node address
    pub fn get_node_address(&self) -> Option<String> {
        self.node_address.clone()
    }
    
    /// Starts both HTTP and WebSocket Ethereum-compatible JSON-RPC servers
    ///
    /// # Arguments
    /// * `http_addr` - The address to bind the HTTP server to
    /// * `ws_addr` - The address to bind the WebSocket server to
    /// * `chain_id` - Chain ID for EIP-155 compatibility
    ///
    /// # Returns
    /// A result containing the combined server instance or an error
    pub async fn start_combined_server(&self, http_addr: &str, ws_addr: &str, chain_id: u64) -> std::result::Result<CombinedServer, JsonRpcError> {
        let http_server = self.start_eth_rpc_server(http_addr, chain_id)?;
        let ws_server = self.start_eth_ws_server(ws_addr, chain_id).await?;

        Ok(CombinedServer {
            http_server,
            ws_server,
        })
    }

    /// Starts an Ethereum-compatible JSON-RPC HTTP server
    ///
    /// # Arguments
    /// * `addr` - The address to bind the server to
    /// * `chain_id` - Chain ID for EIP-155 compatibility
    ///
    /// # Returns
    /// A result containing the server instance or an error
    ///
    /// # Important
    /// The returned server instance must be stored in a variable that lives for the duration
    /// of the program to prevent the Tokio runtime from being dropped in an asynchronous context.
    /// 
    /// # Example
    /// ```
    /// let _eth_server = rpc_handler.start_eth_rpc_server("127.0.0.1:8545", 2030)?;
    /// ```
    /// Note the use of `_eth_server` to store the server instance.
    pub fn start_eth_rpc_server(&self, addr: &str, chain_id: u64) -> std::result::Result<HttpServer, JsonRpcError> {
        let eth_handler = eth_compat::EthRpcHandler::new(self.clone(), chain_id);
        
        // Start the server and return it to be managed by the caller
        eth_handler.start_server(addr).map_err(|_| JsonRpcError::internal_error())
    }

    /// Retrieves account information for a given address
    ///
    /// # Arguments
    /// * `address` - The account address to query
    ///
    /// # Returns
    /// AccountInfo structure containing the account's current state
    ///
    /// # Example
    /// ```
    /// let info = rpc_handler.get_account_info("0x123...".to_string());
    /// println!("Balance: {}", info.balance);
    /// ```
    pub fn get_account_info(&self, address: String) -> AccountInfo {
        // Preserve the original address format for the response
        let original_address = address.clone();
        // Normalize the address for lookup
        let normalized_address = address.to_lowercase();
        
        info!("get_account_info called for address: {}", normalized_address);

        // Query the runtime for account information
        let balance = self.runtime.get_balance(&normalized_address);
        let verified = self.runtime.is_account_verified(&normalized_address);

        info!("Account info retrieved: address={}, balance={}, verified={}", normalized_address, balance, verified);

        // Return the account info with the ORIGINAL address format to maintain case consistency
        AccountInfo {
            address: original_address,
            balance,
            verified,
        }
    }

    /// Creates a new account with the given address
    ///
    /// # Arguments
    /// * `address` - The Ethereum-compatible address for the new account
    ///
    /// # Returns
    /// CreateAccountResponse with success status and account info or error message
    ///
    /// # Example
    /// ```
    /// let response = rpc_handler.create_account("0x1234567890abcdef1234567890abcdef12345678".to_string());
    /// if response.success {
    ///     println!("Account created successfully");
    /// } else {
    ///     println!("Error: {}", response.error.unwrap());
    /// }
    /// ```
    pub fn create_account(&self, address: String) -> CreateAccountResponse {
        let normalized_address = address.to_lowercase();
        match self.runtime.create_account(&normalized_address) {
            Ok(account) => {
                let account_info = AccountInfo {
                    address: account.address,
                    balance: account.balance,
                    verified: account.verified,
                };
                
                CreateAccountResponse {
                    success: true,
                    account: Some(account_info),
                    error: None,
                }
            },
            Err(err) => {
                let error_message = match err {
                    AccountError::AlreadyExists => "Account already exists".to_string(),
                    AccountError::InvalidAddress => "Invalid address format".to_string(),
                    AccountError::Other(msg) => msg,
                };
                
                CreateAccountResponse {
                    success: false,
                    account: None,
                    error: Some(error_message),
                }
            }
        }
    }

    /// Requests tokens from the faucet
    /// 
    /// This function:
    /// 1. Validates the recipient address
    /// 2. Transfers tokens from the faucet to the recipient
    /// 3. Returns the updated balance
    /// 
    /// # Arguments
    /// * `address` - The recipient's address
    /// * `amount` - Optional amount to request (defaults to 10)
    /// 
    /// # Returns
    /// A response indicating success or failure
    pub async fn request_from_faucet(&self, address: String, amount: Option<u64>) -> FaucetResponse {
        let normalized_address = address.to_lowercase();

        if !is_valid_eth_address(&normalized_address) {
            return FaucetResponse {
                success: false,
                amount: None,
                new_balance: None,
                transaction_hash: None,
                error: Some("Invalid Ethereum address".to_string()),
            };
        }

        let faucet_address = match &self.node_address {
            Some(addr) => addr.to_lowercase(),
            None => "0x1111111111111111111111111111111111111111".to_string(),
        };

        let tokens_to_send = amount.unwrap_or(10).min(100);

        let faucet_balance = self.runtime.get_balance(&faucet_address);

        if faucet_balance < tokens_to_send + 1 {
            return FaucetResponse {
                success: false,
                amount: None,
                new_balance: None,
                transaction_hash: None,
                error: Some(format!("Insufficient balance: {} < {}", faucet_balance, tokens_to_send + 1)),
            };
        }

        let recipient_exists = self.runtime.get_balance(&normalized_address) > 0;
        if !recipient_exists {
            match self.runtime.create_account(&normalized_address) {
                Ok(_) => {
                    info!("Created new account for recipient: {}", normalized_address);
                },
                Err(e) => {
                    if let runtime::AccountError::AlreadyExists = e {
                    } else {
                        return FaucetResponse {
                            success: false,
                            amount: None,
                            new_balance: None,
                            transaction_hash: None,
                            error: Some(format!("Failed to create recipient account: {:?}", e)),
                        };
                    }
                }
            }
        }

        // Instead of creating a transaction, directly transfer the tokens
        match self.runtime.transfer_with_fee(&faucet_address, &normalized_address, tokens_to_send) {
            Ok(_) => {
                info!("Faucet transfer successful: {} tokens sent to {}", tokens_to_send, normalized_address);
                
                // Get the updated balance
                let new_balance = self.runtime.get_balance(&normalized_address);
                
                // Generate a transaction hash for compatibility
                let mut tx_hash_bytes = [0u8; 32];
                rand::thread_rng().fill(&mut tx_hash_bytes);
                let tx_hash = format!("0x{}", hex::encode(tx_hash_bytes));
                
                FaucetResponse {
                    success: true,
                    amount: Some(tokens_to_send),
                    new_balance: Some(new_balance),
                    transaction_hash: Some(tx_hash),
                    error: None,
                }
            },
            Err(e) => {
                error!("Faucet transfer failed: {:?}", e);
                FaucetResponse {
                    success: false,
                    amount: None,
                    new_balance: None,
                    transaction_hash: None,
                    error: Some(format!("Failed to transfer tokens: {:?}", e)),
                }
            }
        }
    }

    /// Creates a new faucet transaction
    ///
    /// # Arguments
    /// * `from_address` - The address to send from
    /// * `to_address` - The address to send to
    /// * `amount` - The amount to send
    ///
    /// # Returns
    /// A result containing the transaction hash or an error
    pub async fn create_faucet_transaction(&self, from_address: &str, to_address: &str, amount: u64) -> std::result::Result<String, JsonRpcError> {
        let block_producer = self.runtime.get_block_producer()
            .ok_or_else(|| JsonRpcError::internal_error())?;

        let normalized_from_address = from_address.to_lowercase();
        let normalized_to_address = to_address.to_lowercase();

        let mut tx_hash_bytes = [0u8; 32];
        rand::thread_rng().fill(&mut tx_hash_bytes);
        let tx_hash = format!("0x{}", hex::encode(tx_hash_bytes));

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let transaction = Transaction {
            hash: tx_hash.clone(),
            from: normalized_from_address,
            to: normalized_to_address,
            amount,
            fee: 1,
            timestamp,
        };

        block_producer.submit_transaction(transaction)
            .map_err(|_| JsonRpcError::internal_error())?;

        Ok(tx_hash)
    }

    /// Starts the Ethereum-compatible WebSocket JSON-RPC server with subscription support
    ///
    /// # Arguments
    /// * `addr` - The address to bind the server to
    /// * `chain_id` - Chain ID for EIP-155 compatibility
    ///
    /// # Returns
    /// A result containing the server instance or an error
    pub async fn start_eth_ws_server(&self, addr: &str, chain_id: u64) -> std::result::Result<WsServer, JsonRpcError> {
        let addr = SocketAddr::from_str(addr)
            .map_err(|_| JsonRpcError::internal_error())?;
        
        // Create a standard IoHandler for WebSocket
        let mut io = IoHandler::new();
        
        // Create the PubSub handler
        let pubsub_handler = Arc::new(eth_pubsub::EthPubSubHandler::new(self.clone(), chain_id));
        
        // Create the Ethereum handler
        let eth_handler = Arc::new(eth_compat::EthRpcHandler::new(self.clone(), chain_id));
        
        // Add standard methods
        io.add_method("eth_getBalance", {
            let handler = eth_handler.clone();
            move |params| handler.eth_get_balance(params)
        });
        
        io.add_method("eth_sendTransaction", {
            let handler = eth_handler.clone();
            move |params| handler.eth_send_transaction(params)
        });
        
        io.add_method("eth_getTransactionCount", {
            let handler = eth_handler.clone();
            move |params| handler.eth_get_transaction_count(params)
        });
        
        io.add_method("eth_chainId", {
            let handler = eth_handler.clone();
            move |params| handler.eth_chain_id(params)
        });
        
        io.add_method("eth_blockNumber", {
            let handler = eth_handler.clone();
            move |params| handler.eth_block_number(params)
        });
        
        io.add_method("eth_getBlockByNumber", {
            let handler = eth_handler.clone();
            move |params| handler.eth_get_block_by_number(params)
        });
        
        io.add_method("eth_getBlockByHash", {
            let handler = eth_handler.clone();
            move |params| handler.eth_get_block_by_hash(params)
        });
        
        io.add_method("eth_accounts", {
            let handler = eth_handler.clone();
            move |params| handler.eth_accounts(params)
        });
        
        io.add_method("eth_sendRawTransaction", {
            let handler = eth_handler.clone();
            move |params| handler.eth_send_raw_transaction(params)
        });
        
        // Add WebSocket-specific methods
        io.add_method("eth_subscribe", {
            let handler = pubsub_handler.clone();
            move |params| {
                let handler = handler.clone();
                Box::pin(async move {
                    handler.eth_subscribe(params).await
                })
            }
        });
        
        io.add_method("eth_unsubscribe", {
            let handler = pubsub_handler.clone();
            move |params| {
                let handler = handler.clone();
                Box::pin(async move {
                    handler.eth_unsubscribe(params).await
                })
            }
        });
        
        // Start the WebSocket server
        WsServerBuilder::new(io)
            .max_connections(100)
            .start(&addr)
            .map_err(|_| JsonRpcError::internal_error())
    }

    // TODO: Implement additional RPC methods:
    // - submit_transaction(): Submit a new transaction
    // - claim_ubi(): Process UBI claims
    // - verify_account(): Submit verification proof
    // - get_network_status(): Query network state
    // - request_ai_resources(): Request AI compute allocation
    // - get_verification_status(): Check verification progress
}

/// Validates an Ethereum address
/// 
/// # Arguments
/// * `address` - The address to validate
/// 
/// # Returns
/// `true` if the address is valid, `false` otherwise
pub fn is_valid_eth_address(address: &str) -> bool {
    // Check if the address starts with "0x" and has 42 characters total (0x + 40 hex chars)
    if !address.starts_with("0x") || address.len() != 42 {
        return false;
    }
    
    // Check if the address contains only hexadecimal characters after "0x"
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_info() {
        let runtime = Runtime::new();
        let handler = RpcHandler::new(runtime);
        
        // For a non-existent account
        let info = handler.get_account_info("0x1234567890abcdef1234567890abcdef12345678".to_string());
        assert_eq!(info.balance, 0); // Non-existent accounts have 0 balance
        assert!(!info.verified); // Non-existent accounts are not verified
        
        // Create an account and check its info
        let valid_address = "0x1234567890abcdef1234567890abcdef12345678";
        let _ = handler.create_account(valid_address.to_string());
        
        let info = handler.get_account_info(valid_address.to_string());
        assert_eq!(info.balance, 10); // Initial balance is 10 tokens
        assert!(info.verified); // Accounts are auto-verified
    }
    
    #[test]
    fn test_create_account() {
        let runtime = Runtime::new();
        let handler = RpcHandler::new(runtime);
        
        // Test valid address
        let valid_address = "0x1234567890abcdef1234567890abcdef12345678";
        let response = handler.create_account(valid_address.to_string());
        assert!(response.success);
        assert!(response.account.is_some());
        assert!(response.error.is_none());
        
        let account_info = response.account.unwrap();
        assert_eq!(account_info.address, valid_address);
        assert_eq!(account_info.balance, 10); // Initial balance is 10 tokens
        assert!(account_info.verified); // Accounts are auto-verified
        
        // Test duplicate address
        let duplicate_response = handler.create_account(valid_address.to_string());
        assert!(!duplicate_response.success);
        assert!(duplicate_response.account.is_none());
        assert!(duplicate_response.error.is_some());
        assert_eq!(duplicate_response.error.unwrap(), "Account already exists");
        
        // Test invalid address
        let invalid_address = "invalid_address";
        let invalid_response = handler.create_account(invalid_address.to_string());
        assert!(!invalid_response.success);
        assert!(invalid_response.account.is_none());
        assert!(invalid_response.error.is_some());
        assert_eq!(invalid_response.error.unwrap(), "Invalid address format");
    }
    
    #[tokio::test]
    async fn test_faucet() {
        let runtime = Runtime::new();
        let handler = RpcHandler::new(runtime);
        
        // Test requesting tokens for a new account
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let response = handler.request_from_faucet(address.to_string(), Some(50)).await;
        
        assert!(response.success);
        assert_eq!(response.amount, Some(50));
        assert!(response.new_balance.is_some());
        assert!(response.transaction_hash.is_some());
        assert!(response.error.is_none());
        
        // The account should now have 50 tokens (plus the 10 initial tokens)
        let balance = handler.runtime.get_balance(address);
        assert_eq!(balance, 60);
        
        // Test requesting tokens for an existing account
        let response2 = handler.request_from_faucet(address.to_string(), Some(30)).await;
        
        assert!(response2.success);
        assert_eq!(response2.amount, Some(30));
        assert_eq!(response2.new_balance, Some(90)); // 60 + 30 = 90
        assert!(response2.transaction_hash.is_some());
        assert!(response2.error.is_none());
        
        // Test requesting more than the maximum allowed
        let response3 = handler.request_from_faucet(address.to_string(), Some(200)).await;
        
        assert!(response3.success);
        assert_eq!(response3.amount, Some(100)); // Should be capped at 100
        assert_eq!(response3.new_balance, Some(190)); // 90 + 100 = 190
    }
} 