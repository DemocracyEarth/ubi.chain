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

use runtime::{Runtime, AccountError};
use serde::{Deserialize, Serialize};

// Add Ethereum compatibility module
pub mod eth_compat;

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
        }
    }
    
    /// Starts an Ethereum-compatible JSON-RPC server
    ///
    /// # Arguments
    /// * `addr` - The address to bind the server to (e.g., "127.0.0.1:8545")
    /// * `chain_id` - The chain ID to use for Ethereum compatibility
    ///
    /// # Returns
    /// Result containing the server instance or an error
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
    pub fn start_eth_rpc_server(&self, addr: &str, chain_id: u64) -> Result<jsonrpc_http_server::Server, String> {
        let eth_handler = eth_compat::EthRpcHandler::new(self.clone(), chain_id);
        
        // Start the server and return it to be managed by the caller
        eth_handler.start_server(addr).map_err(|e| format!("Failed to start RPC server: {:?}", e))
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
        // Query the runtime for account information
        let balance = self.runtime.get_balance(&address);
        let verified = self.runtime.is_account_verified(&address);
        
        AccountInfo {
            address,
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
        match self.runtime.create_account(&address) {
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

    /// Handles faucet requests to distribute testnet tokens
    ///
    /// # Arguments
    /// * `address` - The address to send tokens to
    /// * `amount` - Optional amount to request (defaults to 10 tokens if not specified)
    ///
    /// # Returns
    /// A response indicating success or failure
    pub fn request_from_faucet(&self, address: String, amount: Option<u64>) -> FaucetResponse {
        // Default amount is 10 tokens
        let tokens_to_send = amount.unwrap_or(10);
        
        // Maximum amount per request is 100 tokens
        let tokens_to_send = std::cmp::min(tokens_to_send, 100);
        
        // Check if the account exists, create it if it doesn't
        let account_exists = self.runtime.get_balance(&address) > 0 || 
                             self.runtime.is_account_verified(&address);
        
        if !account_exists {
            match self.runtime.create_account(&address) {
                Err(e) => {
                    return FaucetResponse {
                        success: false,
                        amount: None,
                        new_balance: None,
                        error: Some(format!("Failed to create account: {}", e)),
                    };
                },
                Ok(_) => {
                    // Account created successfully
                }
            }
        }
        
        // Create a faucet account if it doesn't exist
        let faucet_address = "0xFAUCET00000000000000000000000000000000000";
        let _ = self.runtime.create_account(faucet_address);
        
        // Transfer tokens from the faucet to the requested address
        match self.runtime.transfer_with_fee(faucet_address, &address, tokens_to_send) {
            Ok(_) => {
                let new_balance = self.runtime.get_balance(&address);
                FaucetResponse {
                    success: true,
                    amount: Some(tokens_to_send),
                    new_balance: Some(new_balance),
                    error: None,
                }
            },
            Err(e) => {
                FaucetResponse {
                    success: false,
                    amount: None,
                    new_balance: None,
                    error: Some(format!("Failed to transfer tokens: {}", e)),
                }
            }
        }
    }

    // TODO: Implement additional RPC methods:
    // - submit_transaction(): Submit a new transaction
    // - claim_ubi(): Process UBI claims
    // - verify_account(): Submit verification proof
    // - get_network_status(): Query network state
    // - request_ai_resources(): Request AI compute allocation
    // - get_verification_status(): Check verification progress
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
    
    #[test]
    fn test_faucet() {
        let runtime = Runtime::new();
        let handler = RpcHandler::new(runtime);
        
        // Test requesting tokens for a new account
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let response = handler.request_from_faucet(address.to_string(), Some(50));
        
        assert!(response.success);
        assert_eq!(response.amount, Some(50));
        assert!(response.new_balance.is_some());
        assert!(response.error.is_none());
        
        // The account should now have 50 tokens (plus the 10 initial tokens)
        let balance = handler.runtime.get_balance(address);
        assert_eq!(balance, 60);
        
        // Test requesting tokens for an existing account
        let response2 = handler.request_from_faucet(address.to_string(), Some(30));
        
        assert!(response2.success);
        assert_eq!(response2.amount, Some(30));
        assert_eq!(response2.new_balance, Some(90)); // 60 + 30 = 90
        assert!(response2.error.is_none());
        
        // Test requesting more than the maximum allowed
        let response3 = handler.request_from_faucet(address.to_string(), Some(200));
        
        assert!(response3.success);
        assert_eq!(response3.amount, Some(100)); // Should be capped at 100
        assert_eq!(response3.new_balance, Some(190)); // 90 + 100 = 190
        assert!(response3.error.is_none());
    }
} 