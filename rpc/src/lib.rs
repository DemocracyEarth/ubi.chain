//! RPC implementation for UBI Chain

use runtime::Runtime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    address: String,
    balance: u64,
    verified: bool,
}

pub struct RpcHandler {
    runtime: Runtime,
}

impl RpcHandler {
    pub fn new() -> Self {
        RpcHandler {
            runtime: Runtime::new(),
        }
    }

    // TODO: Implement RPC methods for interacting with the runtime
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 