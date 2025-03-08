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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// RPC server address
    #[arg(long, default_value = "127.0.0.1:9933")]
    rpc_addr: String,
    
    /// P2P network address
    #[arg(long, default_value = "127.0.0.1:30333")]
    p2p_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    info!("Starting UBI Chain node...");
    
    // Initialize the runtime
    let runtime = Runtime::new();
    
    // Start the P2P network
    let p2p_addr = SocketAddr::from_str(&args.p2p_addr)?;
    let p2p_network = P2PNetwork::new(p2p_addr);
    
    // Start the RPC server in a separate task
    let rpc_addr = args.rpc_addr.clone();
    let rpc_handler = rpc::RpcHandler::new(runtime);
    
    tokio::spawn(async move {
        run_rpc_server(&rpc_addr, rpc_handler).await.unwrap();
    });

    // Connect to some initial peers (for testing/development)
    let initial_peers = vec![
        "127.0.0.1:30334",  // Example peer 1
        "127.0.0.1:30335",  // Example peer 2
    ];

    // Clone p2p_network for the peer monitoring task
    let p2p_network_monitor = p2p_network.clone();
    let monitor_peers = initial_peers.clone();

    // Spawn a task to periodically monitor peer connections
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

    // Try to connect to initial peers
    for peer_addr in &initial_peers {
        if let Ok(addr) = SocketAddr::from_str(peer_addr) {
            info!("Connecting to peer: {}", addr);
            p2p_network.connect_to_peer(addr).await;
        }
    }
    
    // Start the P2P network (this will block)
    p2p_network.start().await?;
    
    Ok(())
}

async fn run_rpc_server(addr: &str, rpc_handler: rpc::RpcHandler) -> Result<(), Box<dyn std::error::Error>> {
    // Bind a TCP listener for the RPC server
    let listener = TcpListener::bind(addr).await?;
    info!("JSON-RPC server listening on {}", addr);

    loop {
        // Accept an incoming TCP connection
        let (mut socket, peer_addr) = listener.accept().await?;
        info!("RPC: Accepted connection from {}", peer_addr);

        // Clone the handler for this connection
        let handler = rpc_handler.clone();

        // Spawn a new task for handling the connection
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            // Read from the socket
            match socket.read(&mut buf).await {
                Ok(0) => return, // Connection closed
                Ok(n) => {
                    // Try to parse the request as a JSON-RPC call
                    if let Ok(request_str) = String::from_utf8(buf[..n].to_vec()) {
                        // Basic example: handle getAccountInfo method
                        if request_str.contains("getAccountInfo") {
                            // Extract address from request (simplified)
                            let response = handler.get_account_info("example_address".to_string());
                            let response_json = serde_json::to_string(&response).unwrap_or_default();
                            if let Err(e) = socket.write_all(response_json.as_bytes()).await {
                                error!("Failed to write response: {:?}", e);
                            }
                        } else {
                            // Echo back for unhandled methods
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