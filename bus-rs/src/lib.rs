use serde::{Deserialize, Serialize};

pub mod listener;
pub mod message;
pub mod message_handler;

pub trait Dep {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    msg_type: String,
    payload: String,
}
