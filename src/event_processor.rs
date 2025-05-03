use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{event::Event, user::User};
use tokio::sync::mpsc::Receiver;

pub async fn process_events(mut rx: Receiver<Event>, user_map: Arc<Mutex<HashMap<String, User>>>) {
    while let Some(event) = rx.recv().await {
        println!(
            "Event #{} from {}: {}",
            event.event_id, event.sender_id, event.event_content
        );
        // TODO: a cycle that gets users with fitting filters
        let mut user_tx = None;

        {
            let mut locked_user_map = user_map.lock().unwrap();
            let user = locked_user_map.get_mut(&event.sender_id);
            match user {
                Some(user) => {
                    user_tx = Some(user.tx.clone());
                }
                None => {}
            }
        }
        match user_tx {
            Some(user_tx) => match user_tx.send(event).await {
                Ok(_) => {}
                Err(_) => {}
            },
            None => {
                continue;
            }
        }
    }
}
