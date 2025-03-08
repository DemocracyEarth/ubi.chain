//! UBI Chain Runtime Implementation
//!
//! This module implements the core blockchain logic including:
//! - Account management and state
//! - Human verification system
//! - UBI distribution mechanisms
//! - AI resource allocation
//! - Transaction execution
//! - State transitions

use std::collections::HashMap;
use std::sync::Arc;

// Add these imports for Ethereum address compatibility
use std::fmt;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    
    #[test]
    fn test_create_account() {
        let runtime = Runtime::new();
        
        // Test valid address
        let valid_address = "0x1234567890abcdef1234567890abcdef12345678";
        let result = runtime.create_account(valid_address);
        assert!(result.is_ok());
        
        let account = result.unwrap();
        assert_eq!(account.address, valid_address);
        assert_eq!(account.balance, 0);
        assert_eq!(account.verified, false);
        
        // Test duplicate address
        let duplicate_result = runtime.create_account(valid_address);
        assert!(duplicate_result.is_err());
        match duplicate_result {
            Err(AccountError::AlreadyExists) => {},
            _ => panic!("Expected AlreadyExists error"),
        }
        
        // Test invalid address (wrong prefix)
        let invalid_prefix = "1x1234567890abcdef1234567890abcdef12345678";
        let invalid_result = runtime.create_account(invalid_prefix);
        assert!(invalid_result.is_err());
        
        // Test invalid address (wrong length)
        let invalid_length = "0x1234567890abcdef1234";
        let invalid_result = runtime.create_account(invalid_length);
        assert!(invalid_result.is_err());
        
        // Test invalid address (non-hex characters)
        let invalid_chars = "0x1234567890abcdef1234567890abcdef1234567z";
        let invalid_result = runtime.create_account(invalid_chars);
        assert!(invalid_result.is_err());
    }
    
    #[test]
    fn test_get_balance() {
        let runtime = Runtime::new();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        
        // Balance should be 0 for non-existent account
        assert_eq!(runtime.get_balance(address), 0);
        
        // Create account and check balance
        let _ = runtime.create_account(address);
        assert_eq!(runtime.get_balance(address), 0);
    }
    
    #[test]
    fn test_is_account_verified() {
        let runtime = Runtime::new();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        
        // Verification status should be false for non-existent account
        assert_eq!(runtime.is_account_verified(address), false);
        
        // Create account and check verification status
        let _ = runtime.create_account(address);
        assert_eq!(runtime.is_account_verified(address), false);
    }
}

/// Error types for account operations
#[derive(Debug)]
pub enum AccountError {
    /// Account already exists with the given address
    AlreadyExists,
    /// Invalid address format
    InvalidAddress,
    /// Other general errors
    Other(String),
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountError::AlreadyExists => write!(f, "Account already exists"),
            AccountError::InvalidAddress => write!(f, "Invalid address format"),
            AccountError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AccountError {}

/// Account structure representing a user in the UBI Chain system
///
/// # Fields
/// * `address` - The unique identifier/address of the account
/// * `balance` - The current balance of UBI tokens
/// * `verified` - Whether the account has passed human verification
///
/// # Example
/// ```
/// let account = Account {
///     address: "0x123...".to_string(),
///     balance: 1000,
///     verified: true
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Account {
    /// Unique identifier for the account (e.g., public key hash)
    pub address: String,
    
    /// Current balance in UBI tokens
    pub balance: u64,
    
    /// Whether the account has passed human verification
    pub verified: bool,
}

/// Runtime implementation for UBI Chain
///
/// The Runtime struct manages the blockchain state and implements
/// core functionality including:
/// - Account state management
/// - Balance tracking
/// - Human verification status
/// - UBI distribution logic
/// - Transaction processing
///
/// # Thread Safety
/// Uses Arc<Mutex<>> for thread-safe state management
#[derive(Clone)]
pub struct Runtime {
    /// Thread-safe storage for account states
    accounts: Arc<std::sync::Mutex<HashMap<String, Account>>>,
}

impl Runtime {
    /// Creates a new Runtime instance with empty state
    ///
    /// # Returns
    /// A new Runtime instance with initialized storage
    pub fn new() -> Self {
        Runtime {
            accounts: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Retrieves the balance for a given account address
    ///
    /// # Arguments
    /// * `address` - The account address to query
    ///
    /// # Returns
    /// The current balance in UBI tokens, or 0 if account doesn't exist
    pub fn get_balance(&self, address: &str) -> u64 {
        self.accounts
            .lock()
            .unwrap()
            .get(address)
            .map(|account| account.balance)
            .unwrap_or(0)
    }

    /// Checks if an account has passed human verification
    ///
    /// # Arguments
    /// * `address` - The account address to check
    ///
    /// # Returns
    /// true if the account exists and is verified, false otherwise
    pub fn is_account_verified(&self, address: &str) -> bool {
        self.accounts
            .lock()
            .unwrap()
            .get(address)
            .map(|account| account.verified)
            .unwrap_or(false)
    }

    /// Creates a new account with the given address
    ///
    /// # Arguments
    /// * `address` - The address for the new account (must be in Ethereum format: 0x + 40 hex chars)
    ///
    /// # Returns
    /// Result with the created account or an error if the account already exists or address is invalid
    ///
    /// # Example
    /// ```
    /// let result = runtime.create_account("0x1234567890abcdef1234567890abcdef12345678");
    /// match result {
    ///     Ok(account) => println!("Created account: {}", account.address),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn create_account(&self, address: &str) -> Result<Account, AccountError> {
        // Validate the address format (should be 0x + 40 hex characters for Ethereum compatibility)
        if !is_valid_eth_address(address) {
            return Err(AccountError::InvalidAddress);
        }
        
        let mut accounts = self.accounts.lock().unwrap();
        
        // Check if account already exists
        if accounts.contains_key(address) {
            return Err(AccountError::AlreadyExists);
        }
        
        // Create new account with zero balance and unverified status
        let account = Account {
            address: address.to_string(),
            balance: 0,
            verified: false,
        };
        
        // Store the account
        accounts.insert(address.to_string(), account.clone());
        
        Ok(account)
    }

    // TODO: Implement the following functionality:
    // - verify_account(): Process human verification
    // - distribute_ubi(): Handle UBI distribution
    // - process_transaction(): Process transfers and other transactions
    // - allocate_ai_resources(): Manage AI resource allocation
    // - update_state(): Handle state transitions
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