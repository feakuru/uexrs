use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{Receiver, Sender};

use crate::amqp::types::frame::Frame;

pub async fn process_frames(
    mut frame_bus_rx: Receiver<Frame>,
    source_map: Arc<Mutex<HashMap<u16, Sender<Frame>>>>,
) {
    while let Some(frame) = frame_bus_rx.recv().await {
        // Bytes 6 and 7 of an AMQP Frame contain the Channel number (see section 2.1 Transport).
        let channel_id = u16::from_be_bytes(frame.type_specific);

        // TODO: match on frame type, parse the frame body and create/update/delete the necessary
        // Connection, Sessions and Links as per the frame type and body (see 2.4-2.7).

        // The frame body is defined as a performative followed by an opaque payload.
        // The performative MUST be one of those defined in section 2.7 Performatives
        // and is encoded as a described type in the AMQP type system.
        // The remaining bytes in the frame body form the payload for that frame.
        // The presence and format of the payload is defined by the semantics
        // of the given performative.

        let mut source: Option<Sender<Frame>> = None;
        {
            let mut locked_source_map = source_map.lock().unwrap();
            if locked_source_map.contains_key(&channel_id) {
                match locked_source_map.get_mut(&channel_id) {
                    Some(good_source) => {
                        source = Some(good_source.clone());
                    }
                    None => {
                        println!("no source");
                        continue;
                    }
                }
            }
        }
        match source {
            Some(source) => match source.send(frame).await {
                Ok(_) => {}
                Err(_) => {
                    println!("could not send frame");
                    continue;
                }
            },
            None => {
                println!("no source");
            }
        }
    }
}
