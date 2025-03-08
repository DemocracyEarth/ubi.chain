/// UBI Chain Node Implementation
/// 
/// This is the main entry point for the UBI Chain node. It implements:
/// - P2P networking for communication with other nodes
/// - JSON-RPC server for external interactions
/// - Runtime execution environment
/// - Chain state management
/// 
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, error};
use std::net::SocketAddr;
use std::str::FromStr;
use clap::Parser;
use serde_json;

mod p2p;
use p2p::P2PNetwork;
use runtime::Runtime;
use rpc;

/// Command line arguments for the node
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// RPC server address for external API access
    /// Default: 127.0.0.1:9933
    #[arg(long, default_value = "127.0.0.1:9933")]
    rpc_addr: String,
    
    /// P2P network address for node communication
    /// Default: 127.0.0.1:30333
    #[arg(long, default_value = "127.0.0.1:30333")]
    p2p_addr: String,
}

/// Main entry point for the UBI Chain node
/// 
/// This function:
/// 1. Initializes the logging system
/// 2. Parses command line arguments
/// 3. Sets up the runtime environment
/// 4. Starts the P2P network
/// 5. Launches the RPC server
/// 6. Manages peer connections
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for debug and monitoring
    env_logger::init();
    
    // Parse command line arguments for configuration
    let args = Args::parse();
    
    info!("Starting UBI Chain node...");
    
    // Initialize the runtime environment for executing chain logic
    let runtime = Runtime::new();
    
    // Start the P2P network for inter-node communication
    let p2p_addr = SocketAddr::from_str(&args.p2p_addr)?;
    let p2p_network = P2PNetwork::new(p2p_addr);
    
    // Initialize and launch the RPC server in a separate task
    let rpc_addr = args.rpc_addr.clone();
    let rpc_handler = rpc::RpcHandler::new(runtime);
    
    tokio::spawn(async move {
        run_rpc_server(&rpc_addr, rpc_handler).await.unwrap();
    });

    // Define initial peers for the network (development/testing)
    let initial_peers = vec![
        "127.0.0.1:30334",  // Example peer 1
        "127.0.0.1:30335",  // Example peer 2
    ];

    // Clone P2P network instance for the peer monitoring task
    let p2p_network_monitor = p2p_network.clone();
    let monitor_peers = initial_peers.clone();

    // Spawn a background task to monitor and maintain peer connections
    tokio::spawn(async move {
        loop {
            for peer_addr in &monitor_peers {
                if let Ok(addr) = SocketAddr::from_str(peer_addr) {
                    // Attempt to reconnect to disconnected peers
                    if !p2p_network_monitor.is_peer_connected(&addr) {
                        info!("Attempting to reconnect to peer: {}", addr);
                        p2p_network_monitor.connect_to_peer(addr).await;
                    }
                }
            }
            // Check connections every minute
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    // Establish initial peer connections
    for peer_addr in &initial_peers {
        if let Ok(addr) = SocketAddr::from_str(peer_addr) {
            info!("Connecting to peer: {}", addr);
            p2p_network.connect_to_peer(addr).await;
        }
    }
    
    // Start the P2P network and block on it
    p2p_network.start().await?;
    
    Ok(())
}

/// RPC server implementation
/// 
/// Handles JSON-RPC requests from external clients including:
/// - Account information queries
/// - Transaction submissions
/// - Chain state queries
/// - Network status information
async fn run_rpc_server(addr: &str, rpc_handler: rpc::RpcHandler) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TCP listener for RPC connections
    let listener = TcpListener::bind(addr).await?;
    info!("JSON-RPC server listening on {}", addr);

    loop {
        // Accept incoming RPC connections
        let (mut socket, peer_addr) = listener.accept().await?;
        info!("RPC: Accepted connection from {}", peer_addr);

        // Clone handler for this connection's task
        let handler = rpc_handler.clone();

        // Spawn a new task for handling this RPC connection
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            // Read incoming RPC request
            match socket.read(&mut buf).await {
                Ok(0) => return, // Connection closed by client
                Ok(n) => {
                    // Parse and handle JSON-RPC request
                    if let Ok(request_str) = String::from_utf8(buf[..n].to_vec()) {
                        // Example: Handle getAccountInfo method
                        if request_str.contains("getAccountInfo") {
                            // Process account information request
                            let response = handler.get_account_info("example_address".to_string());
                            let response_json = serde_json::to_string(&response).unwrap_or_default();
                            if let Err(e) = socket.write_all(response_json.as_bytes()).await {
                                error!("Failed to write response: {:?}", e);
                            }
                        } else {
                            // Echo unhandled methods (temporary)
                            if let Err(e) = socket.write_all(&buf[..n]).await {
                                error!("Failed to write to socket: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from socket: {:?}", e);
                    return;
                }
            }
        });
    }
}