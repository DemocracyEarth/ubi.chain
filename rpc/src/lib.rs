//! RPC implementation for UBI Chain

use runtime::Runtime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    address: String,
    balance: u64,
    verified: bool,
}

#[derive(Clone)]
pub struct RpcHandler {
    runtime: Runtime,
}

impl RpcHandler {
    pub fn new(runtime: Runtime) -> Self {
        RpcHandler {
            runtime,
        }
    }

    pub fn get_account_info(&self, address: String) -> AccountInfo {
        // Use the runtime to fetch account information
        let balance = self.runtime.get_balance(&address);
        let verified = self.runtime.is_account_verified(&address);
        
        AccountInfo {
            address,
            balance,
            verified,
        }
    }

    // TODO: Implement more RPC methods for interacting with the runtime
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 