use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{Router, routing::get};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

mod amqp;
mod event;
mod event_processor;
mod panel;
mod socket_handler;
mod user;

// TODO: proper logging

const READ_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> io::Result<()> {
    // web server stuff
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    // run our app, listening globally on port 3000
    // TODO: get from config
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // pubsub stuff
    // we need to get these params from a config
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    let (pubsub_bus_tx, pubsub_bus_rx) = mpsc::channel(1024);
    let user_map = Arc::new(Mutex::new(HashMap::new()));

    let event_processor_user_map = user_map.clone();
    tokio::spawn(async move {
        event_processor::process_events(pubsub_bus_rx, event_processor_user_map).await;
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        match negotiate_version(&mut socket).await {
            Ok(version) => println!("negotiation successful: version is {}", version),
            Err(_) => {
                println!("negotiation failed");
                continue;
            }
        }
        let pubsub_bus_tx = pubsub_bus_tx.clone();
        let (pubsub_client_tx, pubsub_client_rx) = mpsc::channel(1024);

        let socket_handler_user_map = user_map.clone();
        tokio::spawn(async move {
            let (socket_rcv, socket_snd) = socket.into_split();
            tokio::spawn(async move {
                socket_handler::socket_event_receiver(
                    socket_rcv,
                    pubsub_bus_tx,
                    pubsub_client_tx,
                    socket_handler_user_map,
                )
                .await;
            });
            tokio::spawn(async move {
                socket_handler::socket_event_sender(socket_snd, pubsub_client_rx).await;
            });
        });
    }
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn negotiate_version(socket: &mut TcpStream) -> Result<&'static str, &'static str> {
    let mut valid_version = true;
    for ch in b"AMQP\x00\x01\x00\x00" {
        let buf = socket.read_u8().await.unwrap_or_else(|_| 0);
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
        Ok("1.0.0")
    } else {
        Err("Invalid client protocol version")
    }
}
