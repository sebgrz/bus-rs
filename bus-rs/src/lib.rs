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

pub trait MessageTypeName {
    fn name() -> &'static str;
}

pub trait MessageConstraints: DeserializeOwned + MessageTypeName + 'static {}
impl<T: DeserializeOwned + MessageTypeName + 'static> MessageConstraints for T {}
