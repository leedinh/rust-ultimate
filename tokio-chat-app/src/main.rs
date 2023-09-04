use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8081").await.unwrap();
    let (mut socket, addr) = listener.accept().await.unwrap();
    println!("Accepted connection from {}", addr);

    loop {
        let mut buffer = [0u8; 1024];

        let bytes_read = socket.read(&mut buffer).await.unwrap();
        println!("Received {} bytes from {}", bytes_read, addr);
        socket.write_all(&buffer[..bytes_read]).await.unwrap();
    }
}
