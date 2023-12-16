use serde::de::DeserializeOwned;
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::RawMessage;

pub struct MessageStore {
    messages: Arc<Mutex<HashMap<String, Box<dyn Fn(String) -> Box<dyn Any> + Sync + Send>>>>,
}

impl MessageStore {
    pub fn new() -> Self {
        MessageStore {
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register<TMessage>(&mut self, key: &str)
    where
        TMessage: DeserializeOwned + Send + Sync + 'static,
    {
        let callback: Box<dyn Fn(String) -> Box<dyn Any> + Send + Sync> = Box::new(|msg_payload| {
            let msg: TMessage = serde_json::from_str(&msg_payload).unwrap();
            let msg: Box<dyn Any + Send + Sync> = Box::new(msg);
            return msg;
        });

        self.messages
            .lock()
            .unwrap()
            .insert(key.to_string(), callback);
    }

    pub fn resolve<TMessage>(&self, raw_message: &RawMessage) -> TMessage
    where
        TMessage: DeserializeOwned + Sync + Send + 'static,
    {
        let msg_fn = self.messages.lock().unwrap();
        let msg_fn = msg_fn.get(&raw_message.msg_type).unwrap();
        let msg = msg_fn(raw_message.payload.to_string());
        let msg: Box<TMessage> = msg.downcast::<TMessage>().unwrap();
        return *msg;
    }
}
