use std::collections::HashSet;

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
    incoming_channels: HashSet<u16>,
    outgoing_channels: HashSet<u16>,
    max_frame_size: u32,
    max_channel_number: u16,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Start,
            incoming_channels: HashSet::new(),
            outgoing_channels: HashSet::new(),
            max_frame_size: 512,
            max_channel_number: 0,
        }
    }
}
