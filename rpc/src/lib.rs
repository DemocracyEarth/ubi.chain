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

use runtime::{Runtime, Account, AccountError};
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

/// RPC handler for processing external requests
///
/// This struct provides methods for handling JSON-RPC requests
/// and interacting with the blockchain runtime.
///
/// # Features
/// - Account information queries
/// - Balance checks
/// - Verification status
/// - Transaction processing
/// - UBI claims
/// - AI resource allocation
#[derive(Clone)]
pub struct RpcHandler {
    /// Reference to the blockchain runtime
    runtime: Runtime,
}

impl RpcHandler {
    /// Creates a new RPC handler instance
    ///
    /// # Arguments
    /// * `runtime` - The blockchain runtime instance to handle requests
    ///
    /// # Returns
    /// A new RpcHandler instance connected to the runtime
    pub fn new(runtime: Runtime) -> Self {
        RpcHandler {
            runtime,
        }
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
        
        let info = handler.get_account_info("test_address".to_string());
        assert_eq!(info.balance, 0); // New accounts start with 0 balance
        assert_eq!(info.verified, false); // New accounts start unverified
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
        assert_eq!(account_info.balance, 0);
        assert_eq!(account_info.verified, false);
        
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
} 