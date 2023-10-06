use async_trait::async_trait;
use bus_rs::{ClientCallbackFnAsync, ClientError, RawMessage};
use futures_util::StreamExt as _;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RedisClientAsync {
    pubsub: Option<Box<redis::aio::PubSub>>,
    connection: Option<Arc<Mutex<redis::aio::Connection>>>,
    channel: &'static str,
}

impl RedisClientAsync {
    pub async fn new_receiver(addr: &str, channel: &'static str) -> RedisClientAsync {
        let redis_client = redis::Client::open(addr).unwrap();
        let conn = redis_client
            .get_async_connection()
            .await
            .unwrap()
            .into_pubsub();
        RedisClientAsync {
            pubsub: Some(Box::new(conn)),
            connection: None,
            channel,
        }
    }

    pub async fn new_sender(addr: &str, channel: &'static str) -> RedisClientAsync {
        let redis_client = redis::Client::open(addr).unwrap();
        let conn = redis_client.get_async_connection().await.unwrap();
        RedisClientAsync {
            pubsub: None,
            connection: Some(Arc::new(Mutex::new(conn))),
            channel,
        }
    }
}

#[async_trait]
impl bus_rs::ClientAsync for RedisClientAsync {
    async fn receiver(
        &mut self,
        recv_callback: Arc<ClientCallbackFnAsync>,
    ) -> Result<(), ClientError> {
        if let Some(pubsub) = &mut self.pubsub {
            let _ = pubsub.subscribe(self.channel).await.or_else(|e| {
                if e.is_io_error() {
                    return Err(ClientError::IO(e.to_string()));
                }
                return Err(ClientError::General(e.to_string()));
            });

            let mut pubsub_stream = pubsub.on_message();

            loop {
                let msg = pubsub_stream.next().await.unwrap();
                let raw_message = bus_rs::RawMessage::from(msg.get_payload::<String>().unwrap());
                let _ = recv_callback(raw_message).await;
            }
        }
        Err(ClientError::NotAssignedConnection)
    }

    async fn send(&mut self, msg: &RawMessage) -> Result<(), ClientError> {
        if let Some(connection) = &mut self.connection {
            let mut connection = connection.lock().await;
            let str_msg: String = msg.into();
            let _: i32 = connection
                .publish(self.channel.to_string(), str_msg)
                .await
                .unwrap();

            return Ok(());
        }
        Err(ClientError::NotAssignedConnection)
    }
}
