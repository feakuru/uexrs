use tokio::sync::mpsc::Sender;

use crate::event::Event;

pub const USER_ID_LEN: usize = 16;

pub struct Filter {
    expr: String,
}

pub struct User {
    pub filter: Option<Filter>,
    pub tx: Sender<Event>,
}

impl User {
    pub fn new(filter: Option<Filter>, tx: Sender<Event>) -> Self {
        Self { filter, tx }
    }
}
