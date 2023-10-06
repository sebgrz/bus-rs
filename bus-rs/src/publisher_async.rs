use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{ClientAsync, MessageConstraints, RawMessage};

pub struct PublisherAsync {
    client: Arc<Mutex<dyn ClientAsync + Send + Sync>>,
}

impl PublisherAsync {
    pub fn new(client: Arc<Mutex<dyn ClientAsync + Send + Sync>>) -> Self {
        Self { client }
    }

    // @@@ todo return publisher error
    pub async fn publish<TMessage>(&self, msg: &TMessage)
    where
        TMessage: MessageConstraints,
    {
        let raw_msg = RawMessage {
            msg_type: TMessage::name().to_string(),
            payload: serde_json::to_string(msg).unwrap(),
        };

        let _ = self.client.lock().await.send(&raw_msg).await;
    }
}
