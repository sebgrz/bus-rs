use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub mod builder;
pub mod listener;
pub mod listener_async;
pub mod message_handler;
pub mod message_handler_async;
pub mod message_store;
pub mod publisher;
pub mod publisher_async;

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
    async fn send(&mut self, msg: &RawMessage) -> Result<(), ClientError>;
}

#[derive(Debug)]
pub enum ClientError {
    NotAssignedConnection,
    IO(String),
    General(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawMessage {
    pub msg_type: String,
    pub headers: HashMap<String, String>,
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

pub trait PubSubLayer: Send + Sync {
    fn before(&self, raw_msg: &mut RawMessage);
    fn after(&self, raw_msg: &RawMessage);
}

pub struct PublisherContext {
    pub(crate) client: Box<dyn Client + Send + Sync>,
    pub(crate) layers: Vec<Box<dyn PubSubLayer>>,
}

pub struct PublisherContextAsync {
    pub(crate) client: Box<dyn ClientAsync + Send + Sync>,
    pub(crate) layers: Vec<Box<dyn PubSubLayer>>,
}
