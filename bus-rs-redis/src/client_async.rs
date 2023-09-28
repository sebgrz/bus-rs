use async_trait::async_trait;
use bus_rs::{ClientCallbackFnAsync, ClientError};
use futures_util::StreamExt as _;
use std::sync::Arc;

pub struct RedisClientAsync {
    connection: Box<redis::aio::PubSub>,
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
            connection: Box::new(conn),
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
        let _ = self.connection.subscribe(self.channel).await.or_else(|e| {
            if e.is_io_error() {
                return Err(ClientError::IO(e.to_string()));
            }
            return Err(ClientError::General(e.to_string()));
        });

        let mut pubsub_stream = self.connection.on_message();

        loop {
            let msg = pubsub_stream.next().await.unwrap();
            let raw_message = bus_rs::RawMessage::from(msg.get_payload::<String>().unwrap());
            let _ = recv_callback(raw_message).await;
        }
    }
}
