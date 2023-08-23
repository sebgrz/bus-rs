use serde::{Deserialize, Serialize};

pub mod listener;

pub trait Dep {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message{
    msg_type: String,
    payload: String
}
