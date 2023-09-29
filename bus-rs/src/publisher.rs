use std::sync::{Arc, Mutex};

use crate::{Client, MessageConstraints, RawMessage};

pub struct Publisher {
    client: Arc<Mutex<dyn Client + Send + Sync>>,
}

impl Publisher {
    pub fn new(client: Arc<Mutex<dyn Client + Send + Sync>>) -> Self {
        Self { client }
    }

    // @@@ todo return publisher error
    pub fn publish<TMessage>(&self, msg: &TMessage)
    where
        TMessage: MessageConstraints,
    {
        let raw_msg = RawMessage {
            msg_type: TMessage::name().to_string(),
            payload: serde_json::to_string(msg).unwrap(),
        };

        let _ = self.client.lock().unwrap().send(&raw_msg);
    }
}
