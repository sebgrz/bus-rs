use std::{
    collections::HashMap,
    iter::Iterator,
    sync::{Arc, Mutex},
};

use crate::{Client, MessageConstraints, PubSubLayer, PublisherContext, RawMessage};

pub struct Publisher {
    context: Arc<Mutex<PublisherContext>>,
}

impl Publisher {
    pub fn new(context: Arc<Mutex<PublisherContext>>) -> Self {
        Self { context }
    }

    // @@@ todo return publisher error
    pub fn publish<TMessage>(&self, msg: &TMessage, headers: Option<HashMap<String, String>>)
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
        let mut context = self.context.lock().unwrap();
        context.layers.iter().for_each(|l| {
            l.before(&mut raw_msg);
        });

        let _ = context.client.send(&raw_msg);

        context.layers.iter().rev().for_each(|l| {
            l.after(&raw_msg);
        });
    }
}
