use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod event;
mod event_processor;
mod socket_handler;
mod user;

// TODO: proper logging

#[tokio::main]
async fn main() -> io::Result<()> {
    // we need to get these params from a config
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    let (pubsub_bus_tx, pubsub_bus_rx) = mpsc::channel(1024);
    let user_map = Arc::new(Mutex::new(HashMap::new()));

    let event_processor_user_map = user_map.clone();
    tokio::spawn(async move {
        event_processor::process_events(pubsub_bus_rx, event_processor_user_map).await;
    });

    loop {
        let (socket, socket_addr) = listener.accept().await?;
        let pubsub_bus_tx = pubsub_bus_tx.clone();
        let (pubsub_client_tx, pubsub_client_rx) = mpsc::channel(1024);

        let socket_handler_user_map = user_map.clone();
        tokio::spawn(async move {
            let (socket_rcv, socket_snd) = socket.into_split();
            socket_handler::socket_event_receiver(
                socket_addr.to_string(),
                socket_rcv,
                pubsub_bus_tx,
                pubsub_client_tx,
                socket_handler_user_map,
            )
            .await;
            tokio::spawn(async move {
                socket_handler::socket_event_sender(socket_snd, pubsub_client_rx).await;
            });
        });
    }
}
