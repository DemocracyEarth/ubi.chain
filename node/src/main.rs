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
use log::{info, error, trace, debug, warn};
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
    // Initialize logging with maximum verbosity
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .format_timestamp_millis()
        .init();
    
    trace!("Initializing UBI Chain node...");
    
    // Parse command line arguments for configuration
    let args = Args::parse();
    debug!("Parsed command line arguments: {:?}", args);
    
    info!("Starting UBI Chain node...");
    
    // Initialize the runtime environment for executing chain logic
    trace!("Initializing runtime environment...");
    let runtime = Runtime::new();
    debug!("Runtime environment initialized successfully");
    
    // Calculate RPC port if not specified (P2P port - 20400)
    let rpc_port = args.rpc_port.unwrap_or(args.port - 20400);
    let rpc_addr = format!("{}:{}", args.rpc_host, rpc_port);
    
    info!("P2P network will listen on {}:{}", args.p2p_host, args.port);
    info!("RPC server will listen on {}", rpc_addr);
    
    // Construct P2P address from host and port
    trace!("Setting up P2P network...");
    let p2p_addr = format!("{}:{}", args.p2p_host, args.port);
    let p2p_addr = SocketAddr::from_str(&p2p_addr)?;
    let p2p_network = P2PNetwork::new(p2p_addr);
    debug!("P2P network initialized with address: {}", p2p_addr);
    
    // Initialize and launch the RPC server in a separate task
    trace!("Initializing RPC handler...");
    let rpc_handler = rpc::RpcHandler::new(runtime);
    debug!("RPC handler initialized successfully");
    
    info!("Launching RPC server...");
    tokio::spawn(async move {
        debug!("Starting RPC server on {}", rpc_addr);
        if let Err(e) = run_rpc_server(&rpc_addr, rpc_handler).await {
            error!("RPC server error: {}", e);
            error!("RPC server error details: {:?}", e);
        }
    });

    // Get peers from command line arguments
    let initial_peers = args.peers
        .map(|peers| peers.split(',').map(String::from).collect())
        .unwrap_or_else(Vec::new);

    if initial_peers.is_empty() {
        info!("No initial peers specified. Running in standalone mode.");
    } else {
        info!("Connecting to initial peers: {:?}", initial_peers);
    }

    let p2p_network_monitor = p2p_network.clone();
    let monitor_peers = initial_peers.clone();

    // Only spawn monitoring task if we have peers to monitor
    if !monitor_peers.is_empty() {
        debug!("Starting peer monitoring task for {} peers", monitor_peers.len());
        tokio::spawn(async move {
            loop {
                trace!("Running peer connection check cycle...");
                for peer_addr in &monitor_peers {
                    match SocketAddr::from_str(peer_addr) {
                        Ok(addr) => {
                            let is_connected = p2p_network_monitor.is_peer_connected(&addr);
                            trace!("Peer {} connection status: {}", addr, is_connected);
                            if !is_connected {
                                info!("Attempting to reconnect to peer: {}", addr);
                                p2p_network_monitor.connect_to_peer(addr).await;
                                debug!("Connection attempt completed for peer: {}", addr);
                            }
                        }
                        Err(e) => warn!("Invalid peer address {}: {}", peer_addr, e),
                    }
                }
                trace!("Peer check cycle complete, sleeping for 60 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        // Establish initial peer connections
        debug!("Establishing initial peer connections...");
        for peer_addr in &initial_peers {
            match SocketAddr::from_str(peer_addr) {
                Ok(addr) => {
                    info!("Connecting to peer: {}", addr);
                    p2p_network.connect_to_peer(addr).await;
                    debug!("Initial connection attempt completed for peer: {}", addr);
                }
                Err(e) => warn!("Invalid initial peer address {}: {}", peer_addr, e),
            }
        }
    }
    
    info!("Starting P2P network...");
    trace!("Entering P2P network main loop");
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
    trace!("Initializing RPC server TCP listener...");
    let listener = TcpListener::bind(addr).await?;
    info!("JSON-RPC server listening on {}", addr);

    loop {
        trace!("Waiting for incoming RPC connection...");
        match listener.accept().await {
            Ok((mut socket, peer_addr)) => {
                info!("RPC: Accepted connection from {}", peer_addr);
                let handler = rpc_handler.clone();

                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];
                    trace!("Reading from RPC connection {}", peer_addr);
                    match socket.read(&mut buf).await {
                        Ok(0) => debug!("RPC connection closed by client: {}", peer_addr),
                        Ok(n) => {
                            trace!("Received {} bytes from {}", n, peer_addr);
                            if let Ok(request_str) = String::from_utf8(buf[..n].to_vec()) {
                                debug!("RPC request from {}: {}", peer_addr, request_str);
                                if request_str.contains("getAccountInfo") {
                                    trace!("Processing getAccountInfo request");
                                    let response = handler.get_account_info("example_address".to_string());
                                    let response_json = serde_json::to_string(&response).unwrap_or_default();
                                    debug!("Sending response to {}: {}", peer_addr, response_json);
                                    if let Err(e) = socket.write_all(response_json.as_bytes()).await {
                                        error!("Failed to write response to {}: {:?}", peer_addr, e);
                                    }
                                } else {
                                    debug!("Unhandled RPC method, echoing back");
                                    if let Err(e) = socket.write_all(&buf[..n]).await {
                                        error!("Failed to write to socket {}: {:?}", peer_addr, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from RPC connection {}: {:?}", peer_addr, e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept RPC connection: {:?}", e);
            }
        }
    }
}