use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod connection;
pub mod link;
pub mod session;

pub async fn negotiate_amqp_version(socket: &mut TcpStream) -> Result<&'static str, &'static str> {
    let mut valid_version = true;
    for ch in b"AMQP\x00\x01\x00\x00" {
        let buf = socket.read_u8().await.unwrap_or_else(|_| 255);
        if buf != *ch {
            valid_version = false;
        }
    }
    for ch in b"AMQP\x00\x01\x00\x00" {
        match socket.write_u8(*ch).await {
            Ok(_) => {}
            Err(_) => return Err("Could not write to socket"),
        }
    }
    if valid_version {
        Ok("Client protocol version is 1.0.0")
    } else {
        Err("Invalid client protocol version")
    }
}
