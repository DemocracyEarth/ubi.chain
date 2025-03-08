use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, error};
use std::net::SocketAddr;
use std::str::FromStr;
use clap::Parser;

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
    let rpc_handler = rpc::RpcHandler::new();
    
    tokio::spawn(async move {
        run_rpc_server(&rpc_addr, rpc_handler).await.unwrap();
    });
    
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

        // Spawn a new task for handling the connection
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            // Read from the socket
            match socket.read(&mut buf).await {
                Ok(0) => return, // Connection closed
                Ok(n) => {
                    // Here, you would parse the JSON-RPC request and route it.
                    // For now, we just echo back the received data.
                    if let Err(e) = socket.write_all(&buf[..n]).await {
                        error!("Failed to write to socket: {:?}", e);
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