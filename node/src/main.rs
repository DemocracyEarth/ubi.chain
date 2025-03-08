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
    /// Port number for P2P network communication
    /// Default: 30333
    #[arg(long, default_value = "30333")]
    port: u16,
    
    /// P2P network host address (optional)
    /// Default: 127.0.0.1
    #[arg(long, default_value = "127.0.0.1")]
    p2p_host: String,

    /// Comma-separated list of peer addresses to connect to
    /// Example: --peers 127.0.0.1:30334,127.0.0.1:30335
    #[arg(long)]
    peers: Option<String>,

    /// RPC server host address
    /// Default: 127.0.0.1
    #[arg(long, default_value = "127.0.0.1")]
    rpc_host: String,

    /// RPC server port (optional)
    /// If not specified, will be calculated as (P2P port - 20400)
    /// Example: P2P port 30333 → RPC port 9933
    #[arg(long)]
    rpc_port: Option<u16>,
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
    
    // Calculate RPC port if not specified (P2P port - 20400)
    // This makes P2P port 30333 map to RPC port 9933
    let rpc_port = args.rpc_port.unwrap_or(args.port - 20400);
    let rpc_addr = format!("{}:{}", args.rpc_host, rpc_port);
    
    info!("P2P network will listen on {}:{}", args.p2p_host, args.port);
    info!("RPC server will listen on {}", rpc_addr);
    
    // Construct P2P address from host and port
    let p2p_addr = format!("{}:{}", args.p2p_host, args.port);
    let p2p_addr = SocketAddr::from_str(&p2p_addr)?;
    let p2p_network = P2PNetwork::new(p2p_addr);
    
    // Initialize and launch the RPC server in a separate task
    let rpc_handler = rpc::RpcHandler::new(runtime);
    
    tokio::spawn(async move {
        if let Err(e) = run_rpc_server(&rpc_addr, rpc_handler).await {
            error!("RPC server error: {}", e);
        }
    });

    // Get peers from command line arguments or use empty vec if none specified
    let initial_peers = args.peers
        .map(|peers| peers.split(',').map(String::from).collect())
        .unwrap_or_else(Vec::new);

    if initial_peers.is_empty() {
        info!("No initial peers specified. Running in standalone mode.");
    } else {
        info!("Connecting to initial peers: {:?}", initial_peers);
    }

    // Clone P2P network instance for the peer monitoring task
    let p2p_network_monitor = p2p_network.clone();
    let monitor_peers = initial_peers.clone();

    // Only spawn monitoring task if we have peers to monitor
    if !monitor_peers.is_empty() {
        tokio::spawn(async move {
            loop {
                for peer_addr in &monitor_peers {
                    if let Ok(addr) = SocketAddr::from_str(peer_addr) {
                        if !p2p_network_monitor.is_peer_connected(&addr) {
                            info!("Attempting to reconnect to peer: {}", addr);
                            p2p_network_monitor.connect_to_peer(addr).await;
                        }
                    }
                }
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