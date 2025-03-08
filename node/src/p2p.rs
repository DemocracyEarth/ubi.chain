use log::{info, error};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};

/// Simple peer-to-peer network implementation
#[derive(Clone)]
pub struct P2PNetwork {
    peers: Arc<Mutex<HashMap<SocketAddr, PeerInfo>>>,
    listen_addr: SocketAddr,
}

struct PeerInfo {
    // Add peer metadata as needed
    connected: bool,
}

impl P2PNetwork {
    pub fn new(listen_addr: SocketAddr) -> Self {
        P2PNetwork {
            peers: Arc::new(Mutex::new(HashMap::new())),
            listen_addr,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("P2P network listening on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    info!("New peer connected: {}", peer_addr);
                    self.handle_peer(socket, peer_addr).await;
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_peer(&self, mut socket: TcpStream, addr: SocketAddr) {
        // Add peer to our list
        {
            let mut peers = self.peers.lock().unwrap();
            peers.insert(addr, PeerInfo { connected: true });
        }

        // Spawn a task to handle communication with this peer
        let peers_clone = self.peers.clone();
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            
            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => {
                        // Connection closed
                        break;
                    }
                    Ok(n) => {
                        // Process message (just echo for now)
                        if let Err(e) = socket.write_all(&buffer[..n]).await {
                            error!("Failed to write to socket: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from socket: {}", e);
                        break;
                    }
                }
            }

            // Update peer status when disconnected
            let mut peers = peers_clone.lock().unwrap();
            if let Some(peer_info) = peers.get_mut(&addr) {
                peer_info.connected = false;
            }
            info!("Peer disconnected: {}", addr);
        });
    }

    pub async fn connect_to_peer(&self, addr: SocketAddr) {
        match TcpStream::connect(addr).await {
            Ok(socket) => {
                info!("Successfully connected to peer: {}", addr);
                self.handle_peer(socket, addr).await;
            }
            Err(e) => {
                error!("Failed to connect to peer {}: {}", addr, e);
            }
        }
    }

    pub fn is_peer_connected(&self, addr: &SocketAddr) -> bool {
        if let Some(peer_info) = self.peers.lock().unwrap().get(addr) {
            peer_info.connected
        } else {
            false
        }
    }
}