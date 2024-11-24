use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Echo server is running on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        info!("New Connection: {}", addr);

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
