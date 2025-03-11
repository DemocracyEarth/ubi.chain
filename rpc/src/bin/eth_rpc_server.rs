//! Ethereum-compatible JSON-RPC Server Example
//!
//! This binary demonstrates how to start both HTTP and WebSocket
//! Ethereum-compatible JSON-RPC servers for UBI Chain.

use ubi_chain_rpc::RpcHandler;
use runtime::Runtime;
use std::env;
use log::{info, error, LevelFilter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ctrlc;

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
                // Actually fund the account with 1,000,000 UBI tokens
                match rpc_handler.runtime.credit_balance(&node_address, 1_000_000) {
                    Ok(balance) => info!("Funded node account with 1,000,000 UBI tokens. New balance: {}", balance),
                    Err(e) => error!("Failed to fund node account: {:?}", e),
                }
            },
            Err(e) => error!("Failed to create node account: {:?}", e),
        }
    }
    
    // Create a flag for shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;

    // Start HTTP server
    let http_server = match rpc_handler.start_eth_rpc_server(&http_addr, chain_id) {
        Ok(server) => {
            info!("HTTP server started successfully on {}", http_addr);
            Some(server)
        },
        Err(e) => {
            error!("Failed to start HTTP server: {:?}", e);
            None
        }
    };

    // Start WebSocket server
    let ws_server = match rpc_handler.start_eth_ws_server(&ws_addr, chain_id).await {
        Ok(server) => {
            info!("WebSocket server started successfully on {}", ws_addr);
            Some(server)
        },
        Err(e) => {
            error!("Failed to start WebSocket server: {:?}", e);
            None
        }
    };

    // Wait for shutdown signal
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Clean shutdown
    if let Some(server) = http_server {
        info!("Shutting down HTTP server...");
        drop(server);
    }
    
    if let Some(server) = ws_server {
        info!("Shutting down WebSocket server...");
        drop(server);
    }

    info!("Servers shut down");
    Ok(())
} 