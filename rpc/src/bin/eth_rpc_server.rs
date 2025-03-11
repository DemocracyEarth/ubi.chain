//! Ethereum-compatible JSON-RPC Server Example
//!
//! This binary demonstrates how to start both HTTP and WebSocket
//! Ethereum-compatible JSON-RPC servers for UBI Chain.

use ubi_chain_rpc::{RpcHandler, eth_compat::EthRpcHandler, eth_pubsub::EthPubSubHandler};
use runtime::Runtime;
use std::env;
use log::{info, error, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    // Parse command line arguments
    let http_addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8545".to_string());
    let ws_addr = env::args().nth(2).unwrap_or_else(|| "127.0.0.1:8546".to_string());
    let chain_id = env::args().nth(3).unwrap_or_else(|| "2030".to_string()).parse::<u64>().unwrap_or(2030);

    info!("Starting Ethereum-compatible JSON-RPC servers");
    info!("HTTP server address: {}", http_addr);
    info!("WebSocket server address: {}", ws_addr);
    info!("Chain ID: {}", chain_id);

    // Initialize the runtime
    let runtime = Runtime::new();
    
    // Create the RPC handler
    let mut rpc_handler = RpcHandler::new(runtime);
    
    // Set a node address for faucet operations
    let node_address = "0x0000000000000000000000000000000000000001".to_string();
    rpc_handler.set_node_address(node_address.clone());
    
    // Create the node account if it doesn't exist
    if rpc_handler.runtime.get_balance(&node_address) == 0 {
        match rpc_handler.runtime.create_account(&node_address) {
            Ok(_) => {
                info!("Created node account: {}", node_address);
                // Fund the node account with some initial tokens
                rpc_handler.runtime.mint(&node_address, 1_000_000);
                info!("Funded node account with 1,000,000 UBI tokens");
            },
            Err(e) => error!("Failed to create node account: {:?}", e),
        }
    }
    
    // Start both HTTP and WebSocket servers
    let servers = rpc_handler.start_eth_rpc_servers(&http_addr, &ws_addr, chain_id)
        .map_err(|e| format!("Failed to start servers: {}", e))?;
    
    info!("Servers started successfully");
    info!("Press Ctrl+C to stop the servers");
    
    // Keep the servers running until the process is terminated
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    
    // Handle Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C, shutting down...");
        let _ = tx.send(());
    });
    
    // Wait for shutdown signal
    let _ = rx.await;
    
    // Servers will be dropped when they go out of scope
    info!("Servers shut down");
    
    Ok(())
} 