use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{error, info};

fn setup_logging() {
    tracing_subscriber::fmt::init();
}

#[tokio::main]
async fn main() {
    setup_logging();

    match TcpStream::connect("127.0.0.1:8080").await {
        Ok(mut stream) => {
            // Send message to the server.
            let message = b"hello world";
            stream.write_all(message).await.unwrap();
            info!("Sent: {}", String::from_utf8_lossy(message));

            // Server echos the message back, recieving.
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).await.unwrap();
            info!(
                length = n,
                "Recieved: {}",
                String::from_utf8_lossy(&buffer[0..n])
            );
        }
        Err(e) => error!(error = %e, "Failed to connect to sever."),
    }
}
