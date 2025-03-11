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
use clap::Parser;
use tokio::sync::{mpsc, broadcast};
use tokio::time::{self, Duration, Instant};
use std::sync::Arc;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use runtime::{Runtime, BlockProducer as BlockProducerTrait};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

mod p2p;
use p2p::P2PNetwork;

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
    
    /// Ethereum JSON-RPC server host address
    /// Default: 127.0.0.1
    #[arg(long, default_value = "127.0.0.1")]
    eth_rpc_host: String,

    /// Ethereum JSON-RPC server port
    /// Default: 8545 (standard Ethereum RPC port)
    #[arg(long, default_value = "8545")]
    eth_rpc_port: u16,
    
    /// Chain ID for Ethereum compatibility (EIP-155)
    /// Default: 2030 (UBI Chain network)
    #[arg(long, default_value = "2030")]
    chain_id: u64,
    
    /// Disable Ethereum JSON-RPC server
    #[arg(long)]
    disable_eth_rpc: bool,
}

/// Block structure for the UBI Chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block number/height
    pub number: u64,
    
    /// Hash of the block
    pub hash: String,
    
    /// Hash of the parent block
    pub parent_hash: String,
    
    /// Timestamp when the block was created
    pub timestamp: u64,
    
    /// Transactions included in this block
    pub transactions: Vec<Transaction>,
    
    /// State root hash after applying this block
    pub state_root: String,
    
    /// Block producer identifier
    pub producer: String,
}

/// Transaction structure for the UBI Chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction hash
    pub hash: String,
    
    /// Sender address
    pub from: String,
    
    /// Recipient address
    pub to: String,
    
    /// Amount to transfer
    pub amount: u64,
    
    /// Transaction fee
    pub fee: u64,
    
    /// Timestamp when the transaction was created
    pub timestamp: u64,
}

/// Transaction pool for pending transactions
#[derive(Debug, Clone)]
pub struct TransactionPool {
    /// Pending transactions
    transactions: Arc<std::sync::Mutex<VecDeque<Transaction>>>,
    
    /// Maximum number of transactions per block
    max_txs_per_block: usize,
}

impl TransactionPool {
    /// Creates a new transaction pool
    pub fn new(max_txs_per_block: usize) -> Self {
        TransactionPool {
            transactions: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            max_txs_per_block,
        }
    }
    
    /// Adds a transaction to the pool
    pub fn add_transaction(&self, tx: Transaction) {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.push_back(tx);
    }
    
    /// Gets transactions for the next block
    pub fn get_transactions_for_block(&self) -> Vec<Transaction> {
        let mut transactions = self.transactions.lock().unwrap();
        let mut block_txs = Vec::new();
        
        // Take up to max_txs_per_block transactions
        while !transactions.is_empty() && block_txs.len() < self.max_txs_per_block {
            if let Some(tx) = transactions.pop_front() {
                block_txs.push(tx);
            }
        }
        
        block_txs
    }
    
    /// Gets the number of pending transactions
    pub fn pending_count(&self) -> usize {
        let transactions = self.transactions.lock().unwrap();
        transactions.len()
    }
}

/// Block producer for the UBI Chain
pub struct BlockProducer {
    /// Reference to the blockchain runtime
    runtime: Runtime,
    
    /// Transaction pool
    tx_pool: TransactionPool,
    
    /// Current block number
    current_block: Arc<AtomicU64>,
    
    /// Block time in milliseconds
    block_time_ms: u64,
    
    /// Node identifier (for block producer field)
    node_id: String,
    
    /// Node address (for receiving block rewards)
    node_address: String,
    
    /// Channel for submitting transactions
    tx_sender: broadcast::Sender<Transaction>,
    
    /// Channel for receiving new blocks
    block_sender: mpsc::Sender<Block>,
}

impl BlockProducer {
    /// Creates a new block producer
    pub fn new(
        runtime: Runtime,
        block_time_ms: u64,
        node_id: String,
        node_address: String,
        tx_sender: broadcast::Sender<Transaction>,
        block_sender: mpsc::Sender<Block>,
    ) -> Self {
        // Ensure the node account exists
        match runtime.create_account(&node_address) {
            Ok(_) => debug!("Node account created: {}", node_address),
            Err(_) => debug!("Node account already exists: {}", node_address),
        }
        
        BlockProducer {
            runtime,
            tx_pool: TransactionPool::new(50), // Allow up to 50 transactions per block
            current_block: Arc::new(AtomicU64::new(0)),
            block_time_ms,
            node_id,
            node_address,
            tx_sender,
            block_sender,
        }
    }
    
    /// Starts the block production loop
    pub async fn start(&self) {
        info!("Starting block production with {}ms block time", self.block_time_ms);
        
        // Clone necessary fields for the transaction receiver task
        let tx_pool = self.tx_pool.clone();
        let mut tx_receiver = self.tx_sender.subscribe();
        
        // Spawn a task to receive transactions and add them to the pool
        tokio::spawn(async move {
            while let Ok(tx) = tx_receiver.recv().await {
                debug!("Received transaction: {:?}", tx);
                tx_pool.add_transaction(tx);
            }
        });
        
        // Main block production loop
        loop {
            let start_time = Instant::now();
            
            // Produce a block
            match self.produce_block().await {
                Ok(block) => {
                    info!("Produced block #{} with {} transactions", block.number, block.transactions.len());
                },
                Err(e) => {
                    error!("Failed to produce block: {}", e);
                }
            }
            
            // Calculate how long to sleep to maintain the target block time
            let elapsed = start_time.elapsed();
            let target_duration = Duration::from_millis(self.block_time_ms);
            
            if elapsed < target_duration {
                let sleep_duration = target_duration - elapsed;
                debug!("Block production took {}ms, sleeping for {}ms", 
                       elapsed.as_millis(), sleep_duration.as_millis());
                time::sleep(sleep_duration).await;
            } else {
                warn!("Block production took {}ms, which exceeds the target block time of {}ms",
                      elapsed.as_millis(), self.block_time_ms);
            }
        }
    }
    
    /// Produces a new block with pending transactions
    async fn produce_block(&self) -> Result<Block, String> {
        // Get transactions from the pool
        let pending_transactions = self.tx_pool.get_transactions_for_block();
        let mut successful_transactions = Vec::new();
        
        // Process each transaction
        for tx in pending_transactions {
            match self.runtime.transfer_with_fee(&tx.from, &tx.to, tx.amount) {
                Ok(_) => {
                    info!("Successfully processed transaction: {} -> {}, amount: {}", tx.from, tx.to, tx.amount);
                    successful_transactions.push(tx);
                },
                Err(e) => {
                    error!("Failed to process transaction: {} -> {}, amount: {}, error: {:?}", 
                           tx.from, tx.to, tx.amount, e);
                }
            }
        }
        
        // Get current block number
        let block_number = self.current_block.fetch_add(1, Ordering::SeqCst) + 1;
        
        // Get parent block hash (use a simple hash of the block number for now)
        let parent_hash = format!("0x{:x}", block_number - 1);
        
        // Credit block reward to producer
        match self.runtime.credit_balance(&self.node_address, 100) {
            Ok(new_balance) => {
                info!("Block #{} reward: 100 UBI tokens to {}, new balance: {}", 
                      block_number, self.node_address, new_balance);
            },
            Err(e) => {
                error!("Failed to credit block reward: {:?}", e);
            }
        }
        
        // Create block hash (simple concatenation for now)
        let block_hash = format!("0x{:x}", block_number);
        
        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create the block
        let block = Block {
            number: block_number,
            hash: block_hash,
            parent_hash,
            timestamp,
            transactions: successful_transactions.clone(),
            state_root: "0x0".to_string(), // Simplified for now
            producer: self.node_id.clone(),
        };
        
        // Send block to subscribers
        if let Err(e) = self.block_sender.send(block.clone()).await {
            error!("Failed to broadcast block: {}", e);
        }

        Ok(block)
    }
    
    /// Submits a transaction to the pool
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<(), String> {
        if let Err(e) = self.tx_sender.send(tx) {
            return Err(format!("Failed to submit transaction: {}", e));
        }
        Ok(())
    }
    
    /// Gets the current block number
    pub fn current_block(&self) -> u64 {
        self.current_block.load(Ordering::SeqCst)
    }
}

impl BlockProducerTrait for BlockProducer {
    fn submit_transaction(&self, tx: runtime::Transaction) -> Result<(), String> {
        let node_tx = Transaction {
            hash: tx.hash,
            from: tx.from,
            to: tx.to,
            amount: tx.amount,
            fee: tx.fee,
            timestamp: tx.timestamp,
        };

        // Directly add transaction to the pool
        self.tx_pool.add_transaction(node_tx);
        Ok(())
    }
    
    fn current_block(&self) -> u64 {
        self.current_block.load(Ordering::SeqCst)
    }
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
///
/// # Important
/// 
/// Always run the node with the RUST_LOG environment variable set to avoid runtime issues:
/// ```bash
/// RUST_LOG=info cargo run --bin ubi-chain-node
/// ```
///
/// Possible log levels:
/// - error: Only errors
/// - warn: Warnings and errors
/// - info: Standard information (recommended)
/// - debug: Detailed debugging information
/// - trace: Very verbose logging
///
/// # Examples
///
/// Basic node:
/// ```bash
/// RUST_LOG=info cargo run --bin ubi-chain-node
/// ```
///
/// Custom configuration:
/// ```bash
/// RUST_LOG=info cargo run --bin ubi-chain-node -- --port 30333 --peers 127.0.0.1:30334
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Calculate RPC port if not specified
    let rpc_port = args.rpc_port.unwrap_or(args.port - 20400);
    
    // Create P2P network address
    let p2p_addr = format!("{}:{}", args.p2p_host, args.port);
    let p2p_socket_addr = p2p_addr.parse::<SocketAddr>().expect("Invalid P2P address");
    info!("Starting P2P network on {}", p2p_addr);
    
    // Create RPC server address
    let rpc_addr = format!("{}:{}", args.rpc_host, rpc_port);
    info!("Starting RPC server on {}", rpc_addr);
    
    // Create Ethereum RPC server address
    let eth_rpc_addr = format!("{}:{}", args.eth_rpc_host, args.eth_rpc_port);
    
    // Generate a node address based on the port (for simplicity)
    // In a production environment, this would be a proper Ethereum address
    let node_address = format!("0x{:040x}", args.port);
    info!("Node address: {}", node_address);
    
    // Initialize blockchain runtime with custom checkpoint configuration
    let runtime = Runtime::with_checkpoint_config(
        20, // Keep up to 20 checkpoints
        "./checkpoints" // Use the checkpoints directory in the current working directory
    );
    info!("Initialized blockchain runtime");
    
    // Create RPC handler
    let mut rpc_handler = rpc::RpcHandler::new(runtime.clone());
    
    // Set the node address in the RPC handler
    rpc_handler.set_node_address(node_address.clone());
    info!("Set node address as faucet address: {}", node_address);
    
    // Create channels for transactions and blocks
    let (tx_sender, _) = broadcast::channel(100);
    let (block_sender, mut block_receiver) = mpsc::channel(100);
    
    // Create block producer
    let block_producer = Arc::new(BlockProducer::new(
        runtime.clone(),
        1000, // 1 second block time
        format!("node-{}", args.port),
        node_address.clone(),
        tx_sender,
        block_sender,
    ));
    
    // Set the block producer reference in the runtime
    runtime.set_block_producer(block_producer.clone());
    
    // Start block production
    let block_producer_clone = block_producer.clone();
    tokio::spawn(async move {
        block_producer_clone.start().await;
    });
    
    // Spawn a task to consume blocks from the channel
    tokio::spawn(async move {
        while let Some(block) = block_receiver.recv().await {
            debug!("Received block #{} with {} transactions", block.number, block.transactions.len());
            // In a real implementation, we would process the block here
        }
    });
    
    // Start Ethereum-compatible JSON-RPC server if not disabled
    let _eth_server = if !args.disable_eth_rpc {
        info!("Starting Ethereum-compatible JSON-RPC server on {}", eth_rpc_addr);
        match rpc_handler.start_eth_rpc_server(&eth_rpc_addr, args.chain_id) {
            Ok(server) => {
                info!("Ethereum-compatible JSON-RPC server started successfully");
                Some(server)
            },
            Err(e) => {
                error!("Failed to start Ethereum-compatible JSON-RPC server: {}", e);
                None
            }
        }
    } else {
        info!("Ethereum-compatible JSON-RPC server disabled");
        None
    };
    
    // Start P2P network
    let _p2p_network = P2PNetwork::new(p2p_socket_addr);
    
    // Connect to peers if specified
    if let Some(peers) = args.peers {
        for peer in peers.split(',') {
            if !peer.trim().is_empty() {
                info!("Connecting to peer: {}", peer);
                // In a real implementation, we would connect to the peer here
            }
        }
    }
    
    // Start the standard RPC server
    let rpc_handler_clone = rpc_handler.clone();
    let rpc_addr_clone = rpc_addr.clone();
    tokio::spawn(async move {
        if let Err(e) = run_rpc_server(&rpc_addr_clone, rpc_handler_clone).await {
            error!("RPC server error: {}", e);
        }
    });
    
    // This is a testnet implementation - no mock transactions are generated
    // Users can request tokens from the faucet service via RPC
    info!("UBI Chain testnet node started successfully");
    info!("Faucet service available via RPC endpoint");
    
    // Keep the main task running indefinitely
    // This approach avoids the issue with dropping the runtime in an async context
    std::future::pending::<()>().await;
    
    // This line will never be reached, but we keep it for type correctness
    #[allow(unreachable_code)]
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
                                
                                // Parse the JSON-RPC request
                                let response = if let Ok(request) = serde_json::from_str::<serde_json::Value>(&request_str) {
                                    if let Some(method) = request.get("method").and_then(|m| m.as_str()) {
                                        match method {
                                            "getAccountInfo" => {
                                                trace!("Processing getAccountInfo request");
                                                if let Some(params) = request.get("params").and_then(|p| p.as_array()) {
                                                    if let Some(address) = params.first().and_then(|a| a.as_str()) {
                                                        let response = handler.get_account_info(address.to_string());
                                                        serde_json::to_string(&response).unwrap_or_default()
                                                    } else {
                                                        r#"{"error": "Missing address parameter"}"#.to_string()
                                                    }
                                                } else {
                                                    r#"{"error": "Invalid parameters"}"#.to_string()
                                                }
                                            },
                                            "createAccount" => {
                                                trace!("Processing createAccount request");
                                                if let Some(params) = request.get("params").and_then(|p| p.as_array()) {
                                                    if let Some(address) = params.first().and_then(|a| a.as_str()) {
                                                        let response = handler.create_account(address.to_string());
                                                        serde_json::to_string(&response).unwrap_or_default()
                                                    } else {
                                                        r#"{"error": "Missing address parameter"}"#.to_string()
                                                    }
                                                } else {
                                                    r#"{"error": "Invalid parameters"}"#.to_string()
                                                }
                                            },
                                            "requestFromFaucet" => {
                                                trace!("Processing requestFromFaucet request");
                                                if let Some(params) = request.get("params").and_then(|p| p.as_array()) {
                                                    if let Some(address) = params.first().and_then(|a| a.as_str()) {
                                                        // Get optional amount parameter
                                                        let amount = params.get(1)
                                                            .and_then(|a| a.as_u64());
                                                        
                                                        info!("Faucet request from {}: address={}, amount={:?}", 
                                                             peer_addr, address, amount);
                                                        
                                                        let response = handler.request_from_faucet(address.to_string(), amount).await;
                                                        
                                                        if response.success {
                                                            info!("Faucet request successful: sent {} tokens to {}, new balance: {}",
                                                                 response.amount.unwrap_or(0), address, response.new_balance.unwrap_or(0));
                                                        } else {
                                                            warn!("Faucet request failed: {}", response.error.as_ref().unwrap_or(&String::new()));
                                                        }
                                                        
                                                        serde_json::to_string(&response).unwrap_or_default()
                                                    } else {
                                                        r#"{"error": "Missing address parameter"}"#.to_string()
                                                    }
                                                } else {
                                                    r#"{"error": "Invalid parameters"}"#.to_string()
                                                }
                                            },
                                            _ => {
                                                debug!("Unhandled RPC method: {}", method);
                                                r#"{"error": "Method not found"}"#.to_string()
                                            }
                                        }
                                    } else {
                                        r#"{"error": "Invalid request, missing method"}"#.to_string()
                                    }
                                } else {
                                    r#"{"error": "Invalid JSON-RPC request"}"#.to_string()
                                };
                                
                                debug!("Sending response to {}: {}", peer_addr, response);
                                if let Err(e) = socket.write_all(response.as_bytes()).await {
                                    error!("Failed to write response to {}: {:?}", peer_addr, e);
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