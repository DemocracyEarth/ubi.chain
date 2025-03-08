use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    // Bind a TCP listener on localhost:9933 (the standard Ethereum RPC port)
    let addr = "127.0.0.1:9933";
    let listener = TcpListener::bind(addr).await.expect("Unable to bind TCP listener");
    println!("JSON-RPC server listening on {}", addr);

    loop {
        // Accept an incoming TCP connection
        let (mut socket, peer_addr) = listener.accept().await.expect("Failed to accept connection");
        println!("Accepted connection from {}", peer_addr);

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
                        eprintln!("Failed to write to socket: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {:?}", e);
                    return;
                }
            }
        });
    }
}