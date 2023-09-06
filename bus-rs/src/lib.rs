use serde::{Deserialize, Serialize};

pub mod listener;
pub mod message;
pub mod message_handler;

pub trait Dep {}

pub trait Client {
    fn receiver(&self, recv_callback: &dyn Fn(Message));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: String,
    pub payload: String,
}

pub trait MessageResolver {
    fn name() -> &'static str;
}
