use serde::de::DeserializeOwned;
use std::{any::Any, collections::HashMap};

use crate::RawMessage;

pub struct MessageStore {
    messages: Box<HashMap<String, Box<dyn Fn(Box<&str>) -> Box<dyn Any>>>>,
}

impl MessageStore {
    pub fn new() -> Self {
        MessageStore {
            messages: Box::new(HashMap::new()),
        }
    }

    pub fn register<TMessage>(&mut self, key: &str)
    where
        TMessage: DeserializeOwned + 'static,
    {
        let callback: Box<dyn Fn(Box<&str>) -> Box<dyn Any>> = Box::new(|msg_payload| {
            let msg: TMessage = serde_json::from_str(&msg_payload).unwrap();
            let msg: Box<dyn Any> = Box::new(msg);
            return msg;
        });

        self.messages.insert(key.to_string(), callback);
    }

    pub fn resolve<TMessage>(&self, raw_message: RawMessage) -> TMessage
    where
        TMessage: DeserializeOwned + 'static,
    {
        let msg_fn = self.messages.get(&raw_message.msg_type).unwrap();
        let msg = msg_fn(Box::new(&raw_message.payload));
        let msg: Box<TMessage> = msg.downcast::<TMessage>().unwrap();
        return *msg;
    }
}
