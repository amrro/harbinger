#![allow(dead_code)]
pub mod tcp_flags;

use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

#[derive(Debug, Clone)]
enum ConnectionState {
    Listen,
    SynReceived,
    SynAckSent,
    Established,
    Closed,
}

fn get_flag_for_state(state: &ConnectionState) -> &'static str {
    match state {
        ConnectionState::Listen => "LISTEN",
        ConnectionState::SynReceived => "SYN",
        ConnectionState::SynAckSent => "SYN-ACK",
        ConnectionState::Established => "ACK",
        ConnectionState::Closed => "CLOSED",
    }
}

async fn simulate_handshake(
    addr: &SocketAddr,
    connection_states: Arc<Mutex<HashMap<SocketAddr, ConnectionState>>>,
) {
    let mut states = connection_states.lock().unwrap();

    // Transition: LISTEN → SYN_RECEIVED
    states.insert(*addr, ConnectionState::SynReceived);
    println!(
        "State Transition: LISTEN → SYN_RECEIVED with flag: SYN for {}",
        addr
    );

    // Transition: SYN_RECEIVED → SYN_ACK_SENT
    states.insert(*addr, ConnectionState::SynAckSent);
    println!(
        "State Transition: SYN_RECEIVED → SYN_ACK_SENT with flag: SYN-ACK for {}",
        addr
    );

    // Transition: SYN_ACK_SENT → ESTABLISHED
    states.insert(*addr, ConnectionState::Established);
    println!(
        "State Transition: SYN_ACK_SENT → ESTABLISHED with flag: ACK for {}",
        addr
    );
}

async fn new_conn_info(
    socket: &TcpStream,
    addr: &SocketAddr,
    connection_states: Arc<Mutex<HashMap<SocketAddr, ConnectionState>>>,
) -> io::Result<()> {
    let peer_addr = socket.peer_addr()?;
    let local_addr = socket.local_addr()?;

    // Access the shared connection states
    let mut states = connection_states.lock().unwrap();

    // Initial state: LISTEN
    states.insert(*addr, ConnectionState::Listen);
    println!(
        "State Transition: LISTEN for connection from {}:{} to {}:{}",
        peer_addr.ip(),
        peer_addr.port(),
        local_addr.ip(),
        local_addr.port(),
    );

    // Simulate state transitions
    states.insert(*addr, ConnectionState::SynReceived);
    println!("State Transition: LISTEN → SYN_RECEIVED for {}", addr);

    states.insert(*addr, ConnectionState::Established);
    println!("State Transition: SYN_RECEIVED → ESTABLISHED for {}", addr);

    // Debug log current states
    println!("Current States: {:?}", states);

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Echo server is running on 127.0.0.1:8080");

    // Shared connection states
    let connection_states = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let connection_states = Arc::clone(&connection_states);

        // Simulate handshake and log state transitions
        simulate_handshake(&addr, connection_states).await;

        // Spawn a new task to handle the connection
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = match socket.read(&mut buffer).await {
                    Ok(0) => {
                        info!("Connection closed: {}", addr);
                        break;
                    }
                    Ok(n) => {
                        info!(addr = %addr, length = n, "Data received from the client");
                        n
                    }
                    Err(e) => {
                        error!("Error reading from socket: {}", e);
                        break;
                    }
                };

                if let Err(e) = socket.write_all(&buffer[0..n]).await {
                    error!("Error writing to the socket: {}", e);
                    break;
                }
            }
        });
    }
}
