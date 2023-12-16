use std::{collections::HashMap, sync::Mutex};

use crate::{
    message_handler::MessageHandler, message_store::MessageStore, Client, ClientError,
    MessageConstraints, PubSubLayer, RawMessage,
};

pub struct Listener {
    message_store: Box<MessageStore>,
    client: Box<dyn Client + Send + Sync>,
    handlers: Box<HashMap<String, Box<dyn Fn(&MessageStore, RawMessage) + Send + Sync>>>,
    layers: Box<Vec<Box<dyn PubSubLayer>>>,
}

impl Listener {
    pub fn new(client: Box<dyn Client + Send + Sync>, layers: Vec<Box<dyn PubSubLayer>>) -> Self {
        Listener {
            message_store: Box::new(MessageStore::new()),
            client,
            handlers: Box::new(HashMap::new()),
            layers: Box::new(layers),
        }
    }

    pub fn listen(&mut self) -> Result<(), ClientError> {
        let callback = |msg: RawMessage| {
            Self::handle(msg, &self.layers, &self.handlers, &self.message_store);
        };
        self.client.receiver(&callback)
    }

    pub fn register_handler<TMessage>(
        &mut self,
        handler: impl MessageHandler<TMessage> + Send + Sync + 'static,
    ) where
        TMessage: MessageConstraints + Send + Sync,
    {
        let handler_ref = Mutex::new(handler);
        let handler_fn = move |ms: &MessageStore, data: RawMessage| {
            let headers = match data.headers.is_empty() {
                true => None,
                false => Some(data.headers.clone()),
            };
            let msg = ms.resolve::<TMessage>(&data);
            handler_ref.lock().unwrap().handle(msg, headers);
        };

        self.message_store.register::<TMessage>(TMessage::name());
        self.register_handler_callback::<TMessage, _>(handler_fn);
    }

    pub fn registered_handlers_count(&self) -> usize {
        self.handlers.len()
    }

    fn handle(
        mut msg: RawMessage,
        layers: &Box<Vec<Box<dyn PubSubLayer>>>,
        handlers: &Box<HashMap<String, Box<dyn Fn(&MessageStore, RawMessage) + Send + Sync>>>,
        message_store: &Box<MessageStore>,
    ) {
        layers.iter().for_each(|l| {
            l.before(&mut msg);
        });

        if let Some(handler) = handlers.get(msg.msg_type.as_str()) {
            handler(&message_store, msg.clone());
        }

        layers.iter().rev().for_each(|l| {
            l.after(&msg);
        });
    }

    fn register_handler_callback<TMessage, TCallback>(&mut self, callback: TCallback)
    where
        TMessage: MessageConstraints,
        TCallback: Fn(&MessageStore, RawMessage) + Send + Sync + 'static,
    {
        let callback: Box<dyn Fn(&MessageStore, RawMessage) + Send + Sync> =
            Box::new(move |ms, msg| callback(ms, msg));

        self.handlers.insert(TMessage::name().to_string(), callback);
    }
}
