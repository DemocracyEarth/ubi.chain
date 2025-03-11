//! Ethereum JSON-RPC PubSub Implementation
//!
//! This module implements the Ethereum JSON-RPC PubSub API for WebSocket connections,
//! allowing clients to subscribe to events like new blocks and logs.

use crate::RpcHandler;
use crate::eth_compat::{EthBlock, EthTransaction};
use jsonrpc_core::{Error, Result, Value};
use jsonrpc_pubsub::SubscriptionId;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use log;
use rand::RngCore;
use hex;
use std::sync::Mutex;

/// Subscription types supported by the Ethereum PubSub API
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SubscriptionType {
    /// New block headers
    NewHeads,
    /// New pending transactions
    NewPendingTransactions,
    /// Log events matching a filter
    Logs,
}

impl std::str::FromStr for SubscriptionType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "newHeads" => Ok(SubscriptionType::NewHeads),
            "newPendingTransactions" => Ok(SubscriptionType::NewPendingTransactions),
            "logs" => Ok(SubscriptionType::Logs),
            _ => Err(Error::invalid_params(format!("Invalid subscription type: {}", s))),
        }
    }
}

/// Subscription manager for Ethereum PubSub
pub struct SubscriptionManager {
    /// Map of subscription IDs to subscription types
    subscriptions: RwLock<HashMap<SubscriptionId, SubscriptionType>>,
    /// Reference to the UBI Chain RPC handler
    #[allow(dead_code)]
    rpc_handler: RpcHandler,
}

impl SubscriptionManager {
    /// Creates a new subscription manager
    pub fn new(rpc_handler: RpcHandler) -> Self {
        SubscriptionManager {
            subscriptions: RwLock::new(HashMap::new()),
            rpc_handler,
        }
    }

    /// Adds a new subscription
    pub fn add_subscription(&self, id: SubscriptionId, subscription_type: SubscriptionType) {
        self.subscriptions.write().insert(id.clone(), subscription_type);
        log::info!("Added new subscription: {:?} for type {:?}", id, subscription_type);
    }

    /// Removes a subscription
    pub fn remove_subscription(&self, id: &SubscriptionId) -> bool {
        let removed = self.subscriptions.write().remove(id).is_some();
        if removed {
            log::info!("Removed subscription: {:?}", id);
        }
        removed
    }

    /// Notifies subscribers of a new block
    pub fn notify_new_block(&self, sink: &jsonrpc_pubsub::Sink, block: EthBlock) {
        let block_json = serde_json::to_value(block).unwrap_or(Value::Null);
        
        for (id, sub_type) in self.subscriptions.read().iter() {
            if *sub_type == SubscriptionType::NewHeads {
                let params = jsonrpc_core::Params::Map(serde_json::Map::from_iter([
                    ("subscription".to_string(), Value::String(format!("{:?}", id))),
                    ("result".to_string(), block_json.clone()),
                ]));
                let _ = sink.notify(params);
            }
        }
    }

    /// Notifies subscribers of a new pending transaction
    pub fn notify_new_transaction(&self, sink: &jsonrpc_pubsub::Sink, tx: EthTransaction) {
        let tx_hash = tx.hash.clone();
        
        for (id, sub_type) in self.subscriptions.read().iter() {
            if *sub_type == SubscriptionType::NewPendingTransactions {
                let params = jsonrpc_core::Params::Map(serde_json::Map::from_iter([
                    ("subscription".to_string(), Value::String(format!("{:?}", id))),
                    ("result".to_string(), Value::String(tx_hash.clone())),
                ]));
                let _ = sink.notify(params);
            }
        }
    }
}

#[derive(Clone)]
pub struct Subscription {
    id: String,
    subscription_type: String,
}

impl Subscription {
    pub fn new(id: String, subscription_type: String) -> Self {
        Self {
            id,
            subscription_type,
        }
    }
}

/// Ethereum PubSub handler
pub struct EthPubSubHandler {
    /// Subscription manager
    subscription_manager: Arc<SubscriptionManager>,
    /// Chain ID for EIP-155 compatibility
    #[allow(dead_code)]
    chain_id: u64,
    /// Active subscriptions
    subscriptions: Mutex<HashMap<String, Subscription>>,
}

impl EthPubSubHandler {
    /// Creates a new Ethereum PubSub handler
    pub fn new(rpc_handler: RpcHandler, chain_id: u64) -> Self {
        let subscription_manager = Arc::new(SubscriptionManager::new(rpc_handler));
        
        EthPubSubHandler {
            subscription_manager,
            chain_id,
            subscriptions: Mutex::new(HashMap::new()),
        }
    }

    /// Gets a reference to the subscription manager
    pub fn subscription_manager(&self) -> Arc<SubscriptionManager> {
        self.subscription_manager.clone()
    }

    /// Handles eth_subscribe requests
    pub async fn eth_subscribe(&self, params: jsonrpc_core::Params) -> Result<Value> {
        let params: Vec<Value> = params.parse()?;
        if params.is_empty() {
            return Err(Error::invalid_params("Missing subscription type"));
        }

        let subscription_type = params[0].as_str()
            .ok_or_else(|| Error::invalid_params("Invalid subscription type"))?;

        // Generate a random subscription ID
        let mut rng = rand::thread_rng();
        let mut id_bytes = [0u8; 16];
        rng.fill_bytes(&mut id_bytes);
        let subscription_id = hex::encode(id_bytes);

        match subscription_type {
            "newHeads" => {
                let subscription = Subscription::new(
                    subscription_id.clone(),
                    subscription_type.to_string(),
                );
                
                let mut subscriptions = self.subscriptions.lock().unwrap();
                subscriptions.insert(subscription_id.clone(), subscription);
                
                Ok(Value::String(subscription_id))
            },
            _ => Err(Error::invalid_params("Unsupported subscription type"))
        }
    }

    /// Handles eth_unsubscribe requests
    pub async fn eth_unsubscribe(&self, params: jsonrpc_core::Params) -> Result<Value> {
        let params: Vec<Value> = params.parse()?;
        if params.is_empty() {
            return Err(Error::invalid_params("Missing subscription ID"));
        }

        let subscription_id = params[0].as_str()
            .ok_or_else(|| Error::invalid_params("Invalid subscription ID"))?;

        let mut subscriptions = self.subscriptions.lock().unwrap();
        let removed = subscriptions.remove(subscription_id).is_some();

        Ok(Value::Bool(removed))
    }

    /// Notifies subscribers of a new block
    pub async fn notify_new_heads(&self, block_hash: String, block_number: u64) -> Result<()> {
        let subscriptions = self.subscriptions.lock().unwrap();
        
        for subscription in subscriptions.values() {
            if subscription.subscription_type == "newHeads" {
                log::info!(
                    "New block notification for subscription {}: hash={}, number=0x{:x}",
                    subscription.id, block_hash, block_number
                );
            }
        }
        
        Ok(())
    }
} 