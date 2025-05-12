use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{Router, routing::get};
use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod amqp;
mod event;
mod event_processor;
mod panel;
mod socket_handler;
mod user;

// TODO: proper logging

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
        let (socket, _) = listener.accept().await?;
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
