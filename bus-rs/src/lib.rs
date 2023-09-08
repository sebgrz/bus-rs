use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod listener;
pub mod message_handler;
pub mod message_store;

pub trait Dep {}

pub trait Client {
    fn receiver(&self, recv_callback: &dyn Fn(RawMessage));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawMessage {
    pub msg_type: String,
    pub payload: String,
}

pub trait MessageResolver {
    fn name() -> &'static str;
}

pub trait MessageConstraints: DeserializeOwned + MessageResolver + 'static {}
impl<T: DeserializeOwned + MessageResolver + 'static> MessageConstraints for T {}
