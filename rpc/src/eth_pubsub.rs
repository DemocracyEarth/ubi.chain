//! Ethereum JSON-RPC PubSub Implementation
//!
//! This module implements the Ethereum JSON-RPC PubSub API for WebSocket connections,
//! allowing clients to subscribe to events like new blocks and logs.

use crate::RpcHandler;
use crate::eth_compat::{EthBlock, EthTransaction};
use jsonrpc_core::{Error, Result, Value};
use jsonrpc_pubsub::{Session, SubscriptionId, PubSubHandler, Subscriber};
use serde_json::json;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use log;

/// Subscription types supported by the Ethereum PubSub API
#[derive(Debug, Clone, PartialEq)]
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
        self.subscriptions.write().insert(id, subscription_type);
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
                let _ = sink.notify(id, &block_json);
            }
        }
    }

    /// Notifies subscribers of a new pending transaction
    pub fn notify_new_transaction(&self, sink: &jsonrpc_pubsub::Sink, tx: EthTransaction) {
        let tx_hash = tx.hash.clone();
        
        for (id, sub_type) in self.subscriptions.read().iter() {
            if *sub_type == SubscriptionType::NewPendingTransactions {
                let _ = sink.notify(id, &Value::String(tx_hash.clone()));
            }
        }
    }
}

/// Ethereum PubSub handler
pub struct EthPubSubHandler {
    /// Subscription manager
    subscription_manager: Arc<SubscriptionManager>,
    /// Chain ID for EIP-155 compatibility
    chain_id: u64,
}

impl EthPubSubHandler {
    /// Creates a new Ethereum PubSub handler
    pub fn new(rpc_handler: RpcHandler, chain_id: u64) -> Self {
        let subscription_manager = Arc::new(SubscriptionManager::new(rpc_handler));
        
        EthPubSubHandler {
            subscription_manager,
            chain_id,
        }
    }

    /// Gets a reference to the subscription manager
    pub fn subscription_manager(&self) -> Arc<SubscriptionManager> {
        self.subscription_manager.clone()
    }

    /// Handles eth_subscribe requests
    pub fn eth_subscribe(&self, params: jsonrpc_core::Params, meta: jsonrpc_pubsub::Session, subscriber: Subscriber<Value>) {
        let params: Vec<Value> = match params.parse() {
            Ok(params) => params,
            Err(_) => {
                let _ = subscriber.reject(Error::invalid_params("Invalid parameters"));
                return;
            }
        };

        if params.is_empty() {
            let _ = subscriber.reject(Error::invalid_params("Missing subscription type"));
            return;
        }

        let subscription_type = match params[0].as_str() {
            Some(s) => match s.parse::<SubscriptionType>() {
                Ok(t) => t,
                Err(e) => {
                    let _ = subscriber.reject(e);
                    return;
                }
            },
            None => {
                let _ = subscriber.reject(Error::invalid_params("Invalid subscription type"));
                return;
            }
        };

        // Handle subscription-specific parameters
        match subscription_type {
            SubscriptionType::Logs => {
                // TODO: Implement log filtering
                // For now, we'll accept any log subscription without filtering
            },
            _ => {}
        }

        // Create a subscription
        let sink = meta.sender();
        let subscription_manager = self.subscription_manager.clone();
        
        subscriber.assign_id_async(move |id| {
            subscription_manager.add_subscription(id.clone(), subscription_type);
            Ok(Value::String(format!("{:?}", id)))
        });
    }

    /// Handles eth_unsubscribe requests
    pub fn eth_unsubscribe(&self, params: jsonrpc_core::Params, _meta: jsonrpc_pubsub::Session) -> Result<Value> {
        let params: Vec<SubscriptionId> = params.parse().map_err(|_| Error::invalid_params("Invalid parameters"))?;
        
        if params.is_empty() {
            return Err(Error::invalid_params("Missing subscription ID"));
        }

        let subscription_id = &params[0];
        let removed = self.subscription_manager.remove_subscription(subscription_id);
        
        Ok(Value::Bool(removed))
    }
} 