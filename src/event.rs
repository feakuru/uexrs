use crate::user::USER_ID_LEN;

pub const EVENT_ID_LEN: usize = 16;

#[derive(Debug)]
pub struct Event {
    pub sender_id: String,
    pub event_id: String,
    pub event_content: String,
}

impl Event {
    pub fn new(sender_id: String, event_id: String, event_content: String) -> Self {
        Self {
            sender_id,
            event_id,
            event_content,
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8192);
        buf[..EVENT_ID_LEN].copy_from_slice(self.event_id.as_bytes());
        buf[EVENT_ID_LEN..USER_ID_LEN].copy_from_slice(self.sender_id.as_bytes());
        buf[EVENT_ID_LEN + USER_ID_LEN..].copy_from_slice(self.event_content.as_bytes());
        buf
    }
}
