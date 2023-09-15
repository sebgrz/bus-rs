use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod listener;
pub mod message_handler;
pub mod message_store;

pub trait Dep {}

pub trait Client {
    fn receiver(&mut self, recv_callback: &dyn Fn(RawMessage)) -> Result<(), ClientError>;
}

pub enum ClientError {
    IO(String),
    General(String)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawMessage {
    pub msg_type: String,
    pub payload: String,
}

impl From<String> for RawMessage {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl Into<String> for RawMessage {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

pub trait MessageTypeName {
    fn name() -> &'static str;
}

pub trait MessageConstraints: DeserializeOwned + MessageTypeName + 'static {}
impl<T: DeserializeOwned + MessageTypeName + 'static> MessageConstraints for T {}
