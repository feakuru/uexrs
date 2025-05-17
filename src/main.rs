use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use amqp::transport::negotiate_amqp_version;

mod amqp;
mod frame_bus;
mod panel;
mod terminus_handler;

// TODO: proper logging

#[tokio::main]
async fn main() -> io::Result<()> {
    // web server stuff
    let app = Router::new();

    // run our app, listening globally on port 3000
    // TODO: get from config
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // AMQP stuff
    // we need to get these params from a config
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    let (frame_bus_tx, frame_bus_rx) = mpsc::channel(1024);
    let source_map = Arc::new(Mutex::new(HashMap::new()));

    let frame_bus_source_map = source_map.clone();
    tokio::spawn(async move {
        frame_bus::process_frames(frame_bus_rx, frame_bus_source_map).await;
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        match negotiate_amqp_version(&mut socket).await {
            Ok(version) => println!("negotiation successful: version is {}", version),
            Err(_) => {
                println!("negotiation failed");
                continue;
            }
        }
        let frame_bus_tx = frame_bus_tx.clone();
        let (amqp_client_tx, amqp_client_rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            // TODO: socket timeouts and configurable connection timeouts
            let (socket_rcv, socket_snd) = socket.into_split();
            tokio::spawn(async move {
                terminus_handler::source_handler(socket_snd, amqp_client_rx).await;
            });
            tokio::spawn(async move {
                terminus_handler::target_handler(socket_rcv, frame_bus_tx).await;
            });
        });
    }
}
