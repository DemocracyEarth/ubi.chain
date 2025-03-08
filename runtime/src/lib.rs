//! Runtime implementation for UBI Chain

use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

/// Represents the state of a user account
#[derive(Debug, Clone)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub verified: bool,
}

/// Main runtime implementation
#[derive(Clone)]
pub struct Runtime {
    // TODO: Add state management
    accounts: Arc<std::sync::Mutex<HashMap<String, Account>>>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            accounts: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        self.accounts
            .lock()
            .unwrap()
            .get(address)
            .map(|account| account.balance)
            .unwrap_or(0)
    }

    pub fn is_account_verified(&self, address: &str) -> bool {
        self.accounts
            .lock()
            .unwrap()
            .get(address)
            .map(|account| account.verified)
            .unwrap_or(false)
    }

    // TODO: Add methods for account management, verification, and UBI distribution
} 