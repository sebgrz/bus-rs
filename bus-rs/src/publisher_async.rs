use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{MessageConstraints, PublisherContextAsync, RawMessage};

pub struct PublisherAsync {
    context: Arc<Mutex<PublisherContextAsync>>,
}

impl PublisherAsync {
    pub fn new(context: Arc<Mutex<PublisherContextAsync>>) -> Self {
        Self { context }
    }

    // @@@ todo return publisher error
    pub async fn publish<TMessage>(&self, msg: &TMessage, headers: Option<HashMap<String, String>>)
    where
        TMessage: MessageConstraints,
    {
        let mut raw_msg = RawMessage {
            msg_type: TMessage::name().to_string(),
            headers: match headers {
                Some(h) => h,
                None => HashMap::new(),
            },
            payload: serde_json::to_string(msg).unwrap(),
        };

        let mut context = self.context.lock().await;
        context.layers.iter().for_each(|l| {
            l.before(&mut raw_msg);
        });

        let _ = context.client.send(&raw_msg).await;

        context.layers.iter().rev().for_each(|l| {
            l.after(&raw_msg);
        });
    }
}
