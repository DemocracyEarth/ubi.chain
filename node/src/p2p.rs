use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn connect_to_peer(addr: &str) {
    match TcpStream::connect(addr).await {
        Ok(mut stream) => {
            println!("Connected to peer at {}", addr);
            // Send a simple handshake message.
            let handshake = b"HELLO_PEER";
            stream.write_all(handshake).await.expect("Failed to send handshake");
            
            // Read response (if any)
            let mut buf = vec![0; 1024];
            if let Ok(n) = stream.read(&mut buf).await {
                println!("Received from peer: {}", String::from_utf8_lossy(&buf[..n]));
            }
        },
        Err(e) => eprintln!("Failed to connect to {}: {:?}", addr, e),
    }
}