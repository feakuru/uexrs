use std::collections::HashMap;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::amqp::types::frame::Frame;

enum ConnectionState {
    Start,
    HdrRcvd,
    HdrSent,
    OpenPipe,
    OcPipe,
    OpenRcvd,
    OpenSent,
    ClosePipe,
    Opened,
    CloseRcvd,
    CloseSent,
    Discarding,
    End,
}

struct Connection {
    state: ConnectionState,
    incoming_channels: HashMap<u16, Receiver<Frame>>,
    outgoing_channels: HashMap<u16, Sender<Frame>>,
    max_frame_size: u32,
    max_channel_number: u16,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Start,
            incoming_channels: HashMap::new(),
            outgoing_channels: HashMap::new(),
            max_frame_size: 512,
            max_channel_number: 0,
        }
    }
}
