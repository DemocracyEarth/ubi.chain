//! RPC interface for the UBI Chain.
use runtime::Runtime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    address: String,
    balance: u64,
    verified: bool,
}

pub struct RpcServer {
    runtime: Runtime,
}

impl RpcServer {
    pub fn new(runtime: Runtime) -> Self {
        RpcServer { runtime }
    }

    // TODO: Implement RPC methods for interacting with the runtime
} 