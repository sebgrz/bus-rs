use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;

pub mod listener;
pub mod listener_async;
pub mod message_handler;
pub mod message_handler_async;
pub mod message_store;
pub mod publisher;

pub trait Dep {}

pub trait Client {
    fn receiver(&mut self, recv_callback: &dyn Fn(RawMessage)) -> Result<(), ClientError>;
    fn send(&mut self, msg: &RawMessage) -> Result<(), ClientError>;
}

pub type ClientCallbackFnAsync =
    dyn Fn(RawMessage) -> BoxFuture<'static, Result<(), ClientError>> + Sync + Send;

#[async_trait]
pub trait ClientAsync {
    async fn receiver(
        &mut self,
        recv_callback: Arc<ClientCallbackFnAsync>,
    ) -> Result<(), ClientError>;
}

#[derive(Debug)]
pub enum ClientError {
    IO(String),
    General(String),
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

impl Into<String> for &RawMessage {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

pub trait MessageTypeName {
    fn name() -> &'static str;
}

pub trait MessageConstraints: DeserializeOwned + Serialize + MessageTypeName + 'static {}
impl<T: DeserializeOwned + Serialize + MessageTypeName + 'static> MessageConstraints for T {}
