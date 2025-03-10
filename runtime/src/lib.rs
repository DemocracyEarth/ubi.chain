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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Add these imports for Ethereum address compatibility
use std::fmt;

// Add these imports for checkpoint mechanism
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

// Add these imports for Merkle tree implementation
use sha2::{Sha256, Digest};
use std::collections::VecDeque;

// Constants for UBI distribution
const UBI_TOKENS_PER_HOUR: u64 = 1;

// Constants for the dividend system
const DIVIDEND_PRECISION: u64 = 1_000_000_000; // 10^9 precision for dividend calculations

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
        assert!(account.verified); // Now accounts are auto-verified
        
        // Test duplicate address
        let duplicate_result = runtime.create_account(valid_address);
        assert!(duplicate_result.is_err());
        match duplicate_result {
            Err(AccountError::AlreadyExists) => {},
            _ => panic!("Expected AlreadyExists error"),
        }
        
        // Test invalid address
        let invalid_address = "not-an-eth-address";
        let invalid_result = runtime.create_account(invalid_address);
        assert!(invalid_result.is_err());
        match invalid_result {
            Err(AccountError::InvalidAddress) => {},
            _ => panic!("Expected InvalidAddress error"),
        }
    }
    
    #[test]
    fn test_get_balance() {
        let runtime = Runtime::new();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        
        // Create account
        let _ = runtime.create_account(address);
        
        // Check initial balance
        let balance = runtime.get_balance(address);
        assert_eq!(balance, 0);
    }
    
    #[test]
    fn test_is_account_verified() {
        let runtime = Runtime::new();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        
        // Create account
        let _ = runtime.create_account(address);
        
        // Check verification status
        let verified = runtime.is_account_verified(address);
        assert!(verified);
    }
    
    #[test]
    fn test_ubi_distribution() {
        let runtime = Runtime::new();
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        
        // Create account
        let _ = runtime.create_account(address);
        
        // Manually set the last_ubi_claim to a specific time in the past
        {
            let mut accounts = runtime.accounts.lock().unwrap();
            if let Some(account) = accounts.get_mut(address) {
                // Set last claim to exactly 2 hours ago
                let two_hours_ago = SystemTime::now() - Duration::from_secs(7200);
                account.last_ubi_claim = two_hours_ago;
                
                // Verify initial balance is 0
                assert_eq!(account.balance, 0);
            }
        }
        
        // Call update_ubi_balance
        let new_balance = runtime.update_ubi_balance(address);
        
        // With UBI_TOKENS_PER_HOUR = 1, we should get 2 tokens for 2 hours
        assert_eq!(new_balance, 2, "Expected exactly 2 tokens for 2 hours at 1 token per hour");
        
        // Verify the balance was updated in storage
        let final_balance = runtime.get_balance(address);
        assert_eq!(final_balance, 2);
    }
    
    // New tests for the added functionality
    
    #[test]
    fn test_compute_current_balance() {
        // Test with zero streaming rate
        let account_state = AccountState {
            base_balance: 100,
            last_update: 1000,
            streaming_rate: 0,
        };
        
        let now = 2000;
        let balance = compute_current_balance(&account_state, now);
        assert_eq!(balance, 100); // Should remain unchanged
        
        // Test with positive streaming rate
        let account_state = AccountState {
            base_balance: 100,
            last_update: 1000,
            streaming_rate: 1,
        };
        
        let now = 2000;
        let balance = compute_current_balance(&account_state, now);
        assert_eq!(balance, 1100); // 100 + 1 * (2000 - 1000)
        
        // Test with very large time difference
        let account_state = AccountState {
            base_balance: 100,
            last_update: 1000,
            streaming_rate: 2,
        };
        
        let now = 101000;
        let balance = compute_current_balance(&account_state, now);
        assert_eq!(balance, 200100); // 100 + 2 * (101000 - 1000)
    }
    
    #[test]
    fn test_update_account_state() {
        // Create an account state
        let mut account_state = AccountState {
            base_balance: 100,
            last_update: 1000,
            streaming_rate: 1,
        };
        
        // Update the state
        let now = 2000;
        update_account_state(&mut account_state, now);
        
        // Check that the base balance was updated correctly
        assert_eq!(account_state.base_balance, 1100); // 100 + 1 * (2000 - 1000)
        
        // Check that the last update timestamp was updated
        assert_eq!(account_state.last_update, now);
        
        // Streaming rate should remain unchanged
        assert_eq!(account_state.streaming_rate, 1);
    }
    
    #[test]
    fn test_transfer_with_fee() {
        let runtime = Runtime::new();
        
        // Create sender and recipient accounts
        let sender = "0x1111111111111111111111111111111111111111";
        let recipient = "0x2222222222222222222222222222222222222222";
        
        let _ = runtime.create_account(sender);
        let _ = runtime.create_account(recipient);
        
        // Add some balance to sender
        {
            let mut accounts = runtime.accounts.lock().unwrap();
            if let Some(account) = accounts.get_mut(sender) {
                account.balance = 1000;
            }
        }
        
        // Check initial balances
        let sender_initial = runtime.get_balance(sender);
        let recipient_initial = runtime.get_balance(recipient);
        let fee_pool_initial = runtime.get_fee_pool();
        
        assert_eq!(sender_initial, 1000);
        assert_eq!(recipient_initial, 0);
        assert_eq!(fee_pool_initial, 0);
        
        // Perform transfer with fee
        let transfer_amount = 100;
        let result = runtime.transfer_with_fee(sender, recipient, transfer_amount);
        assert!(result.is_ok());
        
        // Check final balances
        let sender_final = runtime.get_balance(sender);
        let recipient_final = runtime.get_balance(recipient);
        let fee_pool_final = runtime.get_fee_pool();
        
        // Sender should have lost the full amount
        assert_eq!(sender_final, sender_initial - transfer_amount);
        
        // Recipient should have received amount minus 1% fee
        let expected_fee = transfer_amount / 100;
        assert_eq!(recipient_final, recipient_initial + (transfer_amount - expected_fee));
        
        // Fee pool should have received the fee
        assert_eq!(fee_pool_final, fee_pool_initial + expected_fee);
    }
    
    #[test]
    fn test_fee_distribution() {
        let runtime = Runtime::new();
        
        // Create accounts
        let accounts = [
            "0x1111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222",
            "0x3333333333333333333333333333333333333333",
        ];
        
        for &address in &accounts {
            let _ = runtime.create_account(address);
        }
        
        // Set up account balances and total supply
        {
            let mut accounts_map = runtime.accounts.lock().unwrap();
            
            // Account 1: 500 tokens (50%)
            if let Some(account) = accounts_map.get_mut(accounts[0]) {
                account.balance = 500;
            }
            
            // Account 2: 300 tokens (30%)
            if let Some(account) = accounts_map.get_mut(accounts[1]) {
                account.balance = 300;
            }
            
            // Account 3: 200 tokens (20%)
            if let Some(account) = accounts_map.get_mut(accounts[2]) {
                account.balance = 200;
            }
            
            // Set total supply
            *runtime.total_supply.lock().unwrap() = 1000;
        }
        
        // Add some fees to the fee pool
        {
            let mut fee_pool = runtime.fee_pool.lock().unwrap();
            *fee_pool = 100;
        }
        
        // Distribute fees
        let distributed = runtime.distribute_fees();
        assert_eq!(distributed, 100);
        
        // Check that fee pool is now empty
        assert_eq!(*runtime.fee_pool.lock().unwrap(), 0);
        
        // Update and check dividends for each account
        for &address in &accounts {
            runtime.update_account_dividends(address);
        }
        
        // Claim dividends and check balances
        let account1_dividends = runtime.claim_dividends(accounts[0]);
        let account2_dividends = runtime.claim_dividends(accounts[1]);
        let account3_dividends = runtime.claim_dividends(accounts[2]);
        
        // Account 1 should get ~50% of fees
        assert!((49..=51).contains(&account1_dividends));
        
        // Account 2 should get ~30% of fees
        assert!((29..=31).contains(&account2_dividends));
        
        // Account 3 should get ~20% of fees
        assert!((19..=21).contains(&account3_dividends));
        
        // Total distributed should be 100
        assert_eq!(account1_dividends + account2_dividends + account3_dividends, 100);
    }
    
    #[test]
    fn test_merkle_tree() {
        // Create a new Merkle tree
        let mut tree = MerkleTree::new();
        
        // Create some account states
        let account1 = "0x1111111111111111111111111111111111111111";
        let state1 = AccountState {
            base_balance: 100,
            last_update: 1000,
            streaming_rate: 1,
        };
        
        let account2 = "0x2222222222222222222222222222222222222222";
        let state2 = AccountState {
            base_balance: 200,
            last_update: 2000,
            streaming_rate: 2,
        };
        
        // Add accounts to the tree
        tree.update_account(account1, &state1);
        tree.update_account(account2, &state2);
        
        // Get the root hash
        let root_hash = tree.root_hash().unwrap();
        assert_ne!(root_hash, [0u8; 32], "Root hash should not be all zeros");
        
        // Update an account and check that the root hash changes
        let updated_state = AccountState {
            base_balance: 250, // Changed
            last_update: 2500, // Changed
            streaming_rate: 2,
        };
        
        tree.update_account(account2, &updated_state);
        let new_root_hash = tree.root_hash().unwrap();
        
        // Root hash should have changed
        assert_ne!(root_hash, new_root_hash, "Root hash should change after updating an account");
    }
    
    #[test]
    fn test_checkpoint_creation_and_loading() {
        // Use a unique directory for this test to avoid conflicts
        let test_dir = format!("./test_checkpoints_{}", std::process::id());
        
        // Clean up any existing test directory
        let _ = std::fs::remove_dir_all(&test_dir);
        
        // Create a runtime with custom checkpoint config
        let runtime = Runtime::with_checkpoint_config(5, &test_dir);
        
        // Create some accounts
        let accounts = [
            "0x1111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222",
            "0x3333333333333333333333333333333333333333",
        ];
        
        for &address in &accounts {
            let _ = runtime.create_account(address);
        }
        
        // Set up account balances
        {
            let mut accounts_map = runtime.accounts.lock().unwrap();
            
            if let Some(account) = accounts_map.get_mut(accounts[0]) {
                account.balance = 100;
            }
            
            if let Some(account) = accounts_map.get_mut(accounts[1]) {
                account.balance = 200;
            }
            
            if let Some(account) = accounts_map.get_mut(accounts[2]) {
                account.balance = 300;
            }
        }
        
        // Create a checkpoint
        let checkpoint_result = runtime.create_checkpoint(true);
        assert!(checkpoint_result.is_ok(), "Failed to create checkpoint: {:?}", checkpoint_result.err());
        
        let checkpoint = checkpoint_result.unwrap();
        
        // Modify the state
        {
            let mut accounts_map = runtime.accounts.lock().unwrap();
            
            if let Some(account) = accounts_map.get_mut(accounts[0]) {
                account.balance = 999; // Changed
            }
        }
        
        // Check that the balance was changed
        assert_eq!(runtime.get_balance(accounts[0]), 999);
        
        // Load the checkpoint
        let load_result = runtime.load_checkpoint(&checkpoint);
        assert!(load_result.is_ok(), "Failed to load checkpoint: {:?}", load_result.err());
        
        // Check that the balance was restored
        assert_eq!(runtime.get_balance(accounts[0]), 100);
        assert_eq!(runtime.get_balance(accounts[1]), 200);
        assert_eq!(runtime.get_balance(accounts[2]), 300);
        
        // Clean up test files
        let _ = std::fs::remove_dir_all(&test_dir);
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
/// * `last_ubi_claim` - Timestamp of the last UBI claim
///
/// # Example
/// ```
/// let account = Account {
///     address: "0x123...".to_string(),
///     balance: 1000,
///     verified: true,
///     last_ubi_claim: SystemTime::now(),
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
    
    /// Timestamp of the last UBI claim
    pub last_ubi_claim: SystemTime,
}

/// Represents the current state of an account with streaming capabilities
pub struct AccountState {
    /// Base balance of the account in tokens
    pub base_balance: u64,
    
    /// Timestamp of the last update to the account state
    pub last_update: u64,
    
    /// Rate at which tokens are streamed (tokens per time unit)
    pub streaming_rate: u64,
}

/// Computes the current balance of an account based on its base balance, streaming rate,
/// and the time elapsed since the last update.
///
/// # Arguments
/// * `account` - Reference to the account state
/// * `now` - Current timestamp
///
/// # Returns
/// The current balance including streamed tokens
pub fn compute_current_balance(account: &AccountState, now: u64) -> u64 {
    account.base_balance + account.streaming_rate * (now - account.last_update)
}

/// Updates the account state by setting the base balance to the computed current balance
/// and updating the last update timestamp.
///
/// # Arguments
/// * `account` - Mutable reference to the account state to update
/// * `now` - Current timestamp to set as the new last_update
pub fn update_account_state(account: &mut AccountState, now: u64) {
    account.base_balance = compute_current_balance(account, now);
    account.last_update = now;
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
    
    /// Global pool for collected transaction fees
    fee_pool: Arc<std::sync::Mutex<u64>>,
    
    /// Global dividend per token value (scaled by DIVIDEND_PRECISION)
    dividend_per_token: Arc<std::sync::Mutex<u64>>,
    
    /// Total supply of tokens in circulation
    total_supply: Arc<std::sync::Mutex<u64>>,
    
    /// Tracks the last dividend per token value seen by each account
    last_dividend_points: Arc<std::sync::Mutex<HashMap<String, u64>>>,
    
    /// Tracks unclaimed dividends for each account
    unclaimed_dividends: Arc<std::sync::Mutex<HashMap<String, u64>>>,
    
    /// Merkle tree for state verification
    state_tree: Arc<std::sync::Mutex<MerkleTree>>,
    
    /// History of state checkpoints
    checkpoints: Arc<std::sync::Mutex<Vec<StateCheckpoint>>>,
    
    /// Maximum number of checkpoints to keep
    max_checkpoints: usize,
    
    /// Directory to store checkpoint files
    checkpoint_dir: String,
}

/// Represents a checkpoint of the blockchain state
#[derive(Clone, Debug)]
pub struct StateCheckpoint {
    /// Timestamp when the checkpoint was created
    pub timestamp: u64,
    
    /// Merkle root hash at the time of checkpoint
    pub root_hash: [u8; 32],
    
    /// Total number of accounts at checkpoint
    pub account_count: usize,
    
    /// Total supply at checkpoint
    pub total_supply: u64,
    
    /// Fee pool at checkpoint
    pub fee_pool: u64,
    
    /// Path to the checkpoint file
    pub file_path: String,
}

impl Runtime {
    /// Creates a new Runtime instance with empty state
    ///
    /// # Returns
    /// A new Runtime instance with initialized storage
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a new Runtime with custom checkpoint configuration
    ///
    /// # Arguments
    /// * `max_checkpoints` - Maximum number of checkpoints to keep
    /// * `checkpoint_dir` - Directory to store checkpoint files
    ///
    /// # Returns
    /// A new Runtime instance with the specified configuration
    pub fn with_checkpoint_config(max_checkpoints: usize, checkpoint_dir: &str) -> Self {
        let mut runtime = Runtime::new();
        runtime.max_checkpoints = max_checkpoints;
        runtime.checkpoint_dir = checkpoint_dir.to_string();
        
        // Ensure checkpoint directory exists
        if !Path::new(checkpoint_dir).exists() {
            fs::create_dir_all(checkpoint_dir).expect("Failed to create checkpoint directory");
        }
        
        runtime
    }

    /// Retrieves the balance for a given account address
    ///
    /// # Arguments
    /// * `address` - The account address to query
    ///
    /// # Returns
    /// The current balance in UBI tokens, or 0 if account doesn't exist
    pub fn get_balance(&self, address: &str) -> u64 {
        // Update UBI balance before returning
        self.update_ubi_balance(address);
        
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
        
        // Create new account with zero balance and AUTOMATICALLY VERIFIED status (placeholder)
        // Also set the last_ubi_claim to the current time
        let account = Account {
            address: address.to_string(),
            balance: 0,
            verified: true, // Auto-verify all accounts as a placeholder
            last_ubi_claim: SystemTime::now(),
        };
        
        // Store the account
        accounts.insert(address.to_string(), account.clone());
        
        Ok(account)
    }
    
    /// Verifies an account as a human
    ///
    /// # Arguments
    /// * `address` - The account address to verify
    ///
    /// # Returns
    /// true if verification was successful, false if account doesn't exist
    pub fn verify_account(&self, address: &str) -> bool {
        let mut accounts = self.accounts.lock().unwrap();
        
        if let Some(account) = accounts.get_mut(address) {
            account.verified = true;
            true
        } else {
            false
        }
    }
    
    /// Updates the UBI balance for an account based on time elapsed since last claim
    ///
    /// # Arguments
    /// * `address` - The account address to update
    ///
    /// # Returns
    /// The amount of UBI tokens added, or 0 if account doesn't exist or isn't verified
    pub fn update_ubi_balance(&self, address: &str) -> u64 {
        let mut accounts = self.accounts.lock().unwrap();
        
        if let Some(account) = accounts.get_mut(address) {
            // Only distribute UBI to verified accounts
            if account.verified {
                // Calculate hours since last claim
                let now = SystemTime::now();
                let elapsed = now.duration_since(account.last_ubi_claim).unwrap_or(Duration::from_secs(0));
                let hours = elapsed.as_secs() / 3600;
                
                if hours > 0 {
                    // Calculate UBI tokens to add (1 per hour)
                    let tokens_to_add = hours * UBI_TOKENS_PER_HOUR;
                    
                    // Update account
                    account.balance += tokens_to_add;
                    account.last_ubi_claim = now - Duration::from_secs(elapsed.as_secs() % 3600);
                    
                    return tokens_to_add;
                }
            }
        }
        
        0
    }

    /// Distributes the accumulated fees to all token holders proportionally
    /// 
    /// This function:
    /// 1. Calculates the new dividend per token value
    /// 2. Updates the global dividend per token
    /// 3. Resets the fee pool
    /// 
    /// # Returns
    /// The amount of fees distributed
    pub fn distribute_fees(&self) -> u64 {
        let mut fee_pool = self.fee_pool.lock().unwrap();
        let total_supply = *self.total_supply.lock().unwrap();
        
        // If there are no tokens in circulation or no fees to distribute, return 0
        if total_supply == 0 || *fee_pool == 0 {
            return 0;
        }
        
        // Calculate the dividend per token increase
        // Using DIVIDEND_PRECISION to avoid loss of precision in integer division
        let dividend_increase = (*fee_pool * DIVIDEND_PRECISION) / total_supply;
        
        // Update the global dividend per token value
        let mut dividend_per_token = self.dividend_per_token.lock().unwrap();
        *dividend_per_token += dividend_increase;
        
        // Store the distributed amount and reset the fee pool
        let distributed_amount = *fee_pool;
        *fee_pool = 0;
        
        distributed_amount
    }
    
    /// Updates the dividend accounting for a specific account
    /// 
    /// This function:
    /// 1. Calculates the dividends owed to the account since last update
    /// 2. Updates the account's last seen dividend point
    /// 3. Adds the owed dividends to the account's unclaimed dividends
    /// 
    /// # Arguments
    /// * `address` - The account address to update
    /// 
    /// # Returns
    /// The amount of new dividends credited to the account
    pub fn update_account_dividends(&self, address: &str) -> u64 {
        if !is_valid_eth_address(address) {
            return 0;
        }
        
        let accounts = self.accounts.lock().unwrap();
        let account = match accounts.get(address) {
            Some(acc) => acc,
            None => return 0,
        };
        
        let balance = account.balance;
        let current_dividend_per_token = *self.dividend_per_token.lock().unwrap();
        
        // Get the last dividend point seen by this account
        let mut last_points = self.last_dividend_points.lock().unwrap();
        let last_point = *last_points.get(address).unwrap_or(&0);
        
        // Calculate new dividends owed
        let point_diff = current_dividend_per_token - last_point;
        let new_dividends = (balance * point_diff) / DIVIDEND_PRECISION;
        
        // Update the account's last dividend point
        last_points.insert(address.to_string(), current_dividend_per_token);
        drop(last_points); // Release the lock before acquiring a new one
        
        // Add to unclaimed dividends
        if new_dividends > 0 {
            let mut unclaimed = self.unclaimed_dividends.lock().unwrap();
            let current_unclaimed = *unclaimed.get(address).unwrap_or(&0);
            unclaimed.insert(address.to_string(), current_unclaimed + new_dividends);
        }
        
        new_dividends
    }
    
    /// Claims the dividends for an account and adds them to the account balance
    /// 
    /// # Arguments
    /// * `address` - The account address to claim dividends for
    /// 
    /// # Returns
    /// The amount of dividends claimed
    pub fn claim_dividends(&self, address: &str) -> u64 {
        if !is_valid_eth_address(address) {
            return 0;
        }
        
        // First update the account's dividends to ensure all owed dividends are accounted for
        self.update_account_dividends(address);
        
        // Get the unclaimed dividends
        let mut unclaimed = self.unclaimed_dividends.lock().unwrap();
        let to_claim = *unclaimed.get(address).unwrap_or(&0);
        
        if to_claim == 0 {
            return 0;
        }
        
        // Reset unclaimed dividends
        unclaimed.insert(address.to_string(), 0);
        
        // Add to account balance
        let mut accounts = self.accounts.lock().unwrap();
        if let Some(account) = accounts.get_mut(address) {
            account.balance += to_claim;
        }
        
        to_claim
    }
    
    /// Gets the unclaimed dividends for an account
    /// 
    /// # Arguments
    /// * `address` - The account address to check
    /// 
    /// # Returns
    /// The amount of unclaimed dividends
    pub fn get_unclaimed_dividends(&self, address: &str) -> u64 {
        if !is_valid_eth_address(address) {
            return 0;
        }
        
        // First update the account's dividends to ensure all owed dividends are accounted for
        self.update_account_dividends(address);
        
        // Return the unclaimed dividends
        let unclaimed = self.unclaimed_dividends.lock().unwrap();
        *unclaimed.get(address).unwrap_or(&0)
    }
    
    /// Updates the total supply when tokens are minted or burned
    /// This should be called whenever the total supply changes
    /// 
    /// # Arguments
    /// * `amount` - The amount to add (positive) or subtract (negative) from total supply
    /// * `is_addition` - True if adding to supply, false if subtracting
    pub fn update_total_supply(&self, amount: u64, is_addition: bool) {
        let mut total_supply = self.total_supply.lock().unwrap();
        
        if is_addition {
            *total_supply += amount;
        } else {
            // Ensure we don't underflow
            *total_supply = total_supply.saturating_sub(amount);
        }
    }

    /// Transfers tokens from one account to another, deducting a 1% fee
    /// that is added to the global fee pool.
    ///
    /// # Arguments
    /// * `from_address` - The sender's account address
    /// * `to_address` - The recipient's account address
    /// * `amount` - The amount of tokens to transfer
    ///
    /// # Returns
    /// * `Ok(())` if the transfer was successful
    /// * `Err(AccountError)` if the transfer failed
    pub fn transfer_with_fee(&self, from_address: &str, to_address: &str, amount: u64) -> Result<(), AccountError> {
        // Validate addresses
        if !is_valid_eth_address(from_address) || !is_valid_eth_address(to_address) {
            return Err(AccountError::InvalidAddress);
        }
        
        // Calculate fee (1% of the amount)
        let fee = amount / 100;
        let transfer_amount = amount - fee;
        
        // Lock accounts for the transaction
        let mut accounts = self.accounts.lock().unwrap();
        
        // Check if sender exists and has sufficient balance
        let sender = accounts.get_mut(from_address).ok_or_else(|| 
            AccountError::Other(format!("Sender account {} not found", from_address))
        )?;
        
        if sender.balance < amount {
            return Err(AccountError::Other("Insufficient balance for transfer".to_string()));
        }
        
        // Deduct from sender
        sender.balance -= amount;
        
        // Add transfer amount to recipient (create if doesn't exist)
        if let Some(recipient) = accounts.get_mut(to_address) {
            recipient.balance += transfer_amount;
        } else {
            // Create new account for recipient
            let new_account = Account {
                address: to_address.to_string(),
                balance: transfer_amount,
                verified: false,
                last_ubi_claim: SystemTime::now(),
            };
            accounts.insert(to_address.to_string(), new_account);
        }
        
        // Add fee to the global fee pool
        let mut fee_pool = self.fee_pool.lock().unwrap();
        *fee_pool += fee;
        
        Ok(())
    }
    
    /// Gets the current total in the fee pool
    ///
    /// # Returns
    /// The current amount in the fee pool
    pub fn get_fee_pool(&self) -> u64 {
        *self.fee_pool.lock().unwrap()
    }

    /// Creates a checkpoint of the current state
    ///
    /// # Arguments
    /// * `force` - Whether to force checkpoint creation even if no changes since last checkpoint
    ///
    /// # Returns
    /// Result containing the created checkpoint or an error
    pub fn create_checkpoint(&self, force: bool) -> io::Result<StateCheckpoint> {
        // Ensure checkpoint directory exists
        if !Path::new(&self.checkpoint_dir).exists() {
            fs::create_dir_all(&self.checkpoint_dir)?;
        }
        
        // Get current state
        let accounts = self.accounts.lock().unwrap();
        let fee_pool = *self.fee_pool.lock().unwrap();
        let total_supply = *self.total_supply.lock().unwrap();
        
        // Update Merkle tree with current account states
        let mut state_tree = self.state_tree.lock().unwrap();
        for (address, account) in accounts.iter() {
            // Convert Account to AccountState for the Merkle tree
            let account_state = AccountState {
                base_balance: account.balance,
                last_update: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                streaming_rate: 0, // Default to 0 for now
            };
            
            state_tree.update_account(address, &account_state);
        }
        
        // Get root hash
        let root_hash = state_tree.root_hash().unwrap_or([0; 32]);
        
        // Check if we already have a checkpoint with this root hash
        let mut checkpoints = self.checkpoints.lock().unwrap();
        if !force && !checkpoints.is_empty() {
            if let Some(last_checkpoint) = checkpoints.last() {
                if last_checkpoint.root_hash == root_hash {
                    // No changes since last checkpoint
                    return Ok(last_checkpoint.clone());
                }
            }
        }
        
        // Create timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Create checkpoint file path
        let file_path = format!("{}/checkpoint_{}.dat", self.checkpoint_dir, timestamp);
        
        // Serialize state to file
        let mut file = File::create(&file_path)?;
        
        // Write header information
        file.write_all(&timestamp.to_le_bytes())?;
        file.write_all(&root_hash)?;
        file.write_all(&(accounts.len() as u64).to_le_bytes())?;
        file.write_all(&total_supply.to_le_bytes())?;
        file.write_all(&fee_pool.to_le_bytes())?;
        
        // Write account data
        for (address, account) in accounts.iter() {
            // Write address length and address
            let address_bytes = address.as_bytes();
            file.write_all(&(address_bytes.len() as u32).to_le_bytes())?;
            file.write_all(address_bytes)?;
            
            // Write account data
            file.write_all(&account.balance.to_le_bytes())?;
            file.write_all(&(account.verified as u8).to_le_bytes())?;
            
            // Write last UBI claim as seconds since epoch
            let last_claim_secs = account.last_ubi_claim
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            file.write_all(&last_claim_secs.to_le_bytes())?;
        }
        
        // Create checkpoint object
        let checkpoint = StateCheckpoint {
            timestamp,
            root_hash,
            account_count: accounts.len(),
            total_supply,
            fee_pool,
            file_path,
        };
        
        // Add to checkpoints list
        checkpoints.push(checkpoint.clone());
        
        // Prune old checkpoints if we have too many
        self.prune_checkpoints();
        
        Ok(checkpoint)
    }
    
    /// Loads state from a checkpoint
    ///
    /// # Arguments
    /// * `checkpoint` - The checkpoint to load
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load_checkpoint(&self, checkpoint: &StateCheckpoint) -> io::Result<()> {
        let file_path = &checkpoint.file_path;
        let mut file = File::open(file_path)?;
        
        // Read and verify header
        let mut timestamp_bytes = [0u8; 8];
        file.read_exact(&mut timestamp_bytes)?;
        let timestamp = u64::from_le_bytes(timestamp_bytes);
        
        if timestamp != checkpoint.timestamp {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checkpoint timestamp mismatch"
            ));
        }
        
        let mut root_hash = [0u8; 32];
        file.read_exact(&mut root_hash)?;
        
        if root_hash != checkpoint.root_hash {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checkpoint root hash mismatch"
            ));
        }
        
        let mut account_count_bytes = [0u8; 8];
        file.read_exact(&mut account_count_bytes)?;
        let account_count = u64::from_le_bytes(account_count_bytes) as usize;
        
        let mut total_supply_bytes = [0u8; 8];
        file.read_exact(&mut total_supply_bytes)?;
        let total_supply = u64::from_le_bytes(total_supply_bytes);
        
        let mut fee_pool_bytes = [0u8; 8];
        file.read_exact(&mut fee_pool_bytes)?;
        let fee_pool = u64::from_le_bytes(fee_pool_bytes);
        
        // Clear current state
        let mut accounts = self.accounts.lock().unwrap();
        accounts.clear();
        
        *self.fee_pool.lock().unwrap() = fee_pool;
        *self.total_supply.lock().unwrap() = total_supply;
        
        // Reset dividend tracking
        *self.dividend_per_token.lock().unwrap() = 0;
        self.last_dividend_points.lock().unwrap().clear();
        self.unclaimed_dividends.lock().unwrap().clear();
        
        // Read account data
        for _ in 0..account_count {
            // Read address
            let mut address_len_bytes = [0u8; 4];
            file.read_exact(&mut address_len_bytes)?;
            let address_len = u32::from_le_bytes(address_len_bytes) as usize;
            
            let mut address_bytes = vec![0u8; address_len];
            file.read_exact(&mut address_bytes)?;
            let address = String::from_utf8(address_bytes)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in address"))?;
            
            // Read account data
            let mut balance_bytes = [0u8; 8];
            file.read_exact(&mut balance_bytes)?;
            let balance = u64::from_le_bytes(balance_bytes);
            
            let mut verified_bytes = [0u8; 1];
            file.read_exact(&mut verified_bytes)?;
            let verified = verified_bytes[0] != 0;
            
            let mut last_claim_bytes = [0u8; 8];
            file.read_exact(&mut last_claim_bytes)?;
            let last_claim_secs = u64::from_le_bytes(last_claim_bytes);
            
            let last_ubi_claim = UNIX_EPOCH + Duration::from_secs(last_claim_secs);
            
            // Create account
            let account = Account {
                address: address.clone(),
                balance,
                verified,
                last_ubi_claim,
            };
            
            // Add to accounts map
            accounts.insert(address, account);
        }
        
        // Rebuild Merkle tree
        let mut state_tree = self.state_tree.lock().unwrap();
        *state_tree = MerkleTree::new();
        
        for (address, account) in accounts.iter() {
            let account_state = AccountState {
                base_balance: account.balance,
                last_update: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                streaming_rate: 0,
            };
            
            state_tree.update_account(address, &account_state);
        }
        
        Ok(())
    }
    
    /// Prunes old checkpoints to keep storage lean
    fn prune_checkpoints(&self) {
        let mut checkpoints = self.checkpoints.lock().unwrap();
        
        // If we have more checkpoints than the maximum, remove the oldest ones
        while checkpoints.len() > self.max_checkpoints {
            if let Some(oldest) = checkpoints.first().cloned() {
                // Remove from filesystem
                let _ = fs::remove_file(&oldest.file_path);
                
                // Remove from list
                checkpoints.remove(0);
            }
        }
    }
    
    /// Gets a list of all available checkpoints
    ///
    /// # Returns
    /// Vector of available checkpoints
    pub fn list_checkpoints(&self) -> Vec<StateCheckpoint> {
        self.checkpoints.lock().unwrap().clone()
    }
    
    /// Gets the latest checkpoint
    ///
    /// # Returns
    /// Option containing the latest checkpoint, if any
    pub fn latest_checkpoint(&self) -> Option<StateCheckpoint> {
        self.checkpoints.lock().unwrap().last().cloned()
    }
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

/// A node in the Merkle tree
#[derive(Clone, Debug)]
pub struct MerkleNode {
    /// Hash of this node
    pub hash: [u8; 32],
    /// Left child node, if any
    pub left: Option<Box<MerkleNode>>,
    /// Right child node, if any
    pub right: Option<Box<MerkleNode>>,
}

/// A Merkle tree for efficiently storing and verifying account states
#[derive(Clone, Debug)]
pub struct MerkleTree {
    /// Root node of the tree
    pub root: Option<Box<MerkleNode>>,
    /// Mapping of account addresses to their leaf indices
    pub address_indices: HashMap<String, usize>,
    /// Leaf nodes for quick access
    pub leaves: Vec<[u8; 32]>,
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl MerkleNode {
    /// Creates a new leaf node with the given data
    pub fn new_leaf(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        
        MerkleNode {
            hash: hash_array,
            left: None,
            right: None,
        }
    }
    
    /// Creates a new internal node with the given children
    pub fn new_internal(left: Box<MerkleNode>, right: Box<MerkleNode>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&left.hash);
        hasher.update(&right.hash);
        let hash = hasher.finalize();
        
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        
        MerkleNode {
            hash: hash_array,
            left: Some(left),
            right: Some(right),
        }
    }
}

impl MerkleTree {
    /// Creates a new empty Merkle tree
    pub fn new() -> Self {
        MerkleTree {
            root: None,
            address_indices: HashMap::new(),
            leaves: Vec::new(),
        }
    }
    
    /// Serializes an account state into bytes for hashing
    pub fn serialize_account_state(address: &str, state: &AccountState) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Add address
        result.extend_from_slice(address.as_bytes());
        
        // Add base_balance (as 8 bytes)
        result.extend_from_slice(&state.base_balance.to_le_bytes());
        
        // Add last_update (as 8 bytes)
        result.extend_from_slice(&state.last_update.to_le_bytes());
        
        // Add streaming_rate (as 8 bytes)
        result.extend_from_slice(&state.streaming_rate.to_le_bytes());
        
        result
    }
    
    /// Adds or updates an account state in the tree
    pub fn update_account(&mut self, address: &str, state: &AccountState) {
        let serialized = Self::serialize_account_state(address, state);
        let leaf_hash = MerkleNode::new_leaf(&serialized).hash;
        
        if let Some(index) = self.address_indices.get(address) {
            // Update existing leaf
            self.leaves[*index] = leaf_hash;
        } else {
            // Add new leaf
            let index = self.leaves.len();
            self.leaves.push(leaf_hash);
            self.address_indices.insert(address.to_string(), index);
        }
        
        // Rebuild the tree
        self.rebuild();
    }
    
    /// Rebuilds the Merkle tree from the leaves
    fn rebuild(&mut self) {
        if self.leaves.is_empty() {
            self.root = None;
            return;
        }
        
        // Create leaf nodes
        let mut nodes: VecDeque<Box<MerkleNode>> = self.leaves
            .iter()
            .map(|hash| {
                Box::new(MerkleNode {
                    hash: *hash,
                    left: None,
                    right: None,
                })
            })
            .collect();
        
        // If odd number of nodes, duplicate the last one
        if nodes.len() % 2 == 1 {
            nodes.push_back(nodes.back().unwrap().clone());
        }
        
        // Build the tree bottom-up
        while nodes.len() > 1 {
            let mut new_level = VecDeque::new();
            
            while !nodes.is_empty() {
                let left = nodes.pop_front().unwrap();
                
                // If we have an odd number of nodes at this level
                if nodes.is_empty() {
                    new_level.push_back(left);
                    break;
                }
                
                let right = nodes.pop_front().unwrap();
                let parent = Box::new(MerkleNode::new_internal(left, right));
                new_level.push_back(parent);
            }
            
            // If odd number of nodes in the new level, duplicate the last one
            if new_level.len() % 2 == 1 && new_level.len() > 1 {
                new_level.push_back(new_level.back().unwrap().clone());
            }
            
            nodes = new_level;
        }
        
        self.root = if nodes.is_empty() { None } else { Some(nodes.pop_front().unwrap()) };
    }
    
    /// Gets the Merkle root hash
    pub fn root_hash(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|node| node.hash)
    }
    
    /// Generates a Merkle proof for the given account address
    pub fn generate_proof(&self, address: &str) -> Option<Vec<([u8; 32], bool)>> {
        let index = self.address_indices.get(address)?;
        let mut proof = Vec::new();
        let mut current_index = *index;
        
        // Start from the leaf level and work up to the root
        let mut level_size = self.leaves.len();
        let mut level_start = 0;
        
        while level_size > 1 {
            let sibling_index = if current_index % 2 == 0 {
                // Current node is left child, sibling is right
                current_index + 1
            } else {
                // Current node is right child, sibling is left
                current_index - 1
            };
            
            // Ensure sibling index is valid
            if sibling_index < level_start + level_size {
                let is_right_sibling = current_index % 2 == 0;
                
                if sibling_index < self.leaves.len() {
                    proof.push((self.leaves[sibling_index], is_right_sibling));
                }
            }
            
            // Move up to parent level
            current_index = level_start + (current_index - level_start) / 2;
            level_size = (level_size + 1) / 2;
            level_start += level_size;
        }
        
        Some(proof)
    }
    
    /// Verifies a Merkle proof for the given account state
    pub fn verify_proof(
        root_hash: [u8; 32],
        address: &str,
        state: &AccountState,
        proof: &[([u8; 32], bool)]
    ) -> bool {
        let serialized = Self::serialize_account_state(address, state);
        let mut current_hash = MerkleNode::new_leaf(&serialized).hash;
        
        for &(sibling_hash, is_right) in proof {
            let mut hasher = Sha256::new();
            
            if is_right {
                // Sibling is on the right
                hasher.update(current_hash);
                hasher.update(sibling_hash);
            } else {
                // Sibling is on the left
                hasher.update(sibling_hash);
                hasher.update(current_hash);
            }
            
            let hash = hasher.finalize();
            current_hash.copy_from_slice(&hash);
        }
        
        current_hash == root_hash
    }
}

// Add Default implementation for Runtime
impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            accounts: Arc::new(std::sync::Mutex::new(HashMap::new())),
            fee_pool: Arc::new(std::sync::Mutex::new(0)),
            dividend_per_token: Arc::new(std::sync::Mutex::new(0)),
            total_supply: Arc::new(std::sync::Mutex::new(0)),
            last_dividend_points: Arc::new(std::sync::Mutex::new(HashMap::new())),
            unclaimed_dividends: Arc::new(std::sync::Mutex::new(HashMap::new())),
            state_tree: Arc::new(std::sync::Mutex::new(MerkleTree::new())),
            checkpoints: Arc::new(std::sync::Mutex::new(Vec::new())),
            max_checkpoints: 10, // Default to keeping 10 checkpoints
            checkpoint_dir: "./checkpoints".to_string(),
        }
    }
} 