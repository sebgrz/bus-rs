use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    message_handler::MessageHandler, message_store::MessageStore, Client, ClientError,
    MessageConstraints, RawMessage,
};

pub struct Listener {
    message_store: Arc<Mutex<MessageStore>>,
    client: Arc<Mutex<dyn Client + Send + Sync>>,
    handlers: Box<HashMap<String, Box<dyn Fn(&MessageStore, RawMessage) + Send + Sync>>>,
}

impl Listener {
    pub fn new(client: Arc<Mutex<dyn Client + Send + Sync>>) -> Self {
        Listener {
            message_store: Arc::new(Mutex::new(MessageStore::new())),
            client,
            handlers: Box::new(HashMap::new()),
        }
    }

    pub fn listen(&mut self) -> Result<(), ClientError> {
        let callback = |msg: RawMessage| {
            self.handle(msg);
        };
        self.client.lock().unwrap().receiver(&callback)
    }

    pub fn register_handler<TMessage>(
        &mut self,
        handler: impl MessageHandler<TMessage> + Send + Sync + 'static,
    ) where
        TMessage: MessageConstraints + Send + Sync,
    {
        let handler_ref = Mutex::new(handler);
        let handler_fn = move |ms: &MessageStore, data: RawMessage| {
            let msg = ms.resolve::<TMessage>(data);
            handler_ref.lock().unwrap().handle(msg);
        };

        self.message_store
            .lock()
            .unwrap()
            .register::<TMessage>(TMessage::name());
        self.register_handler_callback::<TMessage, _>(handler_fn);
    }

    pub fn registered_handlers_count(&self) -> usize {
        self.handlers.len()
    }

    fn handle(&self, msg: RawMessage) {
        if let Some(handler) = self.handlers.get(msg.msg_type.as_str()) {
            handler(&self.message_store.lock().unwrap(), msg);
        }
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
