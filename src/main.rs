pub mod tcp_flags;
pub mod tcp_headers;

use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

async fn new_conn_info(socket: &TcpStream, addr: &SocketAddr) -> io::Result<()> {
    let peer_addr = socket.peer_addr()?;
    let local_addr = socket.local_addr()?;

    info!(
        "New Connection: {} \nTCP Headers:\n\tSource Port: {} \n\tDestination Port: {}",
        addr,
        peer_addr.port(),
        local_addr.port(),
    );

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Echo server is running on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        new_conn_info(&socket, &addr).await?;

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = match socket.read(&mut buffer).await {
                    Ok(0) => {
                        info!("Connectino closed: {}", addr);
                        break;
                    }
                    Ok(n) => {
                        info!(addr = %addr, length = n, "Date received from the client");
                        n
                    }
                    Err(e) => {
                        error!("Error reading from socket: {}", e);
                        break;
                    }
                };

                if let Err(e) = socket.write_all(&buffer[0..n]).await {
                    error!("Erorr writing to the socket: {}", e);
                    break;
                }
            }
        });
    }
}
