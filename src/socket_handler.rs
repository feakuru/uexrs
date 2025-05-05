use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::event::{EVENT_ID_LEN, Event};
use crate::user::{USER_ID_LEN, User};

pub async fn socket_event_receiver<R>(
    mut socket_reader: R,
    pubsub_tx: Sender<Event>,
    pubsub_client_tx: Sender<Event>,
    user_map: Arc<Mutex<HashMap<String, User>>>,
) where
    R: AsyncReadExt + Unpin + Send + 'static,
{
    let mut buf = vec![0; 8192];

    loop {
        match socket_reader.read(&mut buf).await {
            // Return value of `Ok(0)` signifies that the remote has closed
            Ok(0) => return,
            Ok(n) => {
                if EVENT_ID_LEN > n {
                    continue;
                }
                let mut event_id = vec![];
                event_id.extend_from_slice(&buf[..EVENT_ID_LEN]);
                let event_id =
                    String::from_utf8(event_id).unwrap_or(String::from("invalid_event_id"));
                let mut user_id = vec![];
                user_id.extend_from_slice(&buf[EVENT_ID_LEN..EVENT_ID_LEN + USER_ID_LEN]);
                let user_id = String::from_utf8(user_id)
                    .unwrap_or(String::from("{\"error\": \"parse_content\"}"));
                let mut event_content = vec![];
                event_content.extend_from_slice(&buf[EVENT_ID_LEN + USER_ID_LEN..n]);
                let event_content = String::from_utf8(event_content)
                    .unwrap_or(String::from("{\"error\": \"parse_content\"}"));
                user_map.lock().unwrap().insert(
                    // we should check if it exists already
                    user_id.clone(),
                    User::new(None, pubsub_client_tx.clone()),
                );
                if pubsub_tx
                    .send(Event::new(user_id, event_id, event_content))
                    .await
                    .is_err()
                {
                    // Well, let's die.
                    return;
                }
            }
            Err(_) => {
                // Unexpected socket error. There isn't much we can do
                // here so just stop processing.
                return;
            }
        }
    }
}

pub async fn socket_event_sender<W>(mut socket: W, mut pubsub_rx: Receiver<Event>)
where
    W: AsyncWriteExt + Unpin + Send + 'static,
{
    loop {
        let event = pubsub_rx.recv().await;
        match event {
            Some(event) => {
                if socket.write(event.to_bytes().as_slice()).await.is_err() {
                    return;
                }
            }
            None => {
                return;
            }
        }
    }
}
