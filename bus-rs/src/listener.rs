use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::de::DeserializeOwned;

use crate::{
    message::MessageStore,
    message_handler::{message_handler_dispatcher, MessageHandler},
    Client, Dep, Message, MessageResolver,
};

pub struct Listener {
    message_store: Rc<RefCell<MessageStore>>,
    client: Box<dyn Client>,
    dep: Box<dyn Dep>,
    handlers: Box<HashMap<String, Box<dyn Fn(&MessageStore, Message)>>>,
}

impl Listener {
    pub fn listen(&mut self) {
        let callback = |msg: Message| {
            message_handler_dispatcher(msg);
        };
        self.client.receiver(&callback);
    }

    pub fn register_handler<TMessage>(&mut self, handler: impl MessageHandler<TMessage> + 'static)
    where
        TMessage: DeserializeOwned + MessageResolver + 'static,
    {
        let handler_ref = RefCell::new(handler);
        let handler_fn = move |ms: &MessageStore, data: Message| {
            let msg = ms.resolve::<TMessage>(data);
            handler_ref.borrow_mut().handle(msg);
        };

        self.message_store
            .borrow_mut()
            .register::<TMessage>(TMessage::name());
        self.register_handler_callback::<TMessage, _>(handler_fn);
    }

    pub fn handle(&self, msg: Message) {
        if let Some(handler) = self.handlers.get(msg.msg_type.as_str()) {
            handler(&self.message_store.borrow(), msg);
        }
    }

    pub fn registered_handlers_count(&self) -> usize {
        self.handlers.len()
    }

    fn register_handler_callback<TMessage, TCallback>(&mut self, callback: TCallback)
    where
        TMessage: DeserializeOwned + MessageResolver + 'static,
        TCallback: Fn(&MessageStore, Message) + 'static,
    {
        let callback: Box<dyn Fn(&MessageStore, Message)> =
            Box::new(move |ms, msg| callback(ms, msg));

        self.handlers.insert(TMessage::name().to_string(), callback);
    }
}

pub fn create_listener(
    message_store: MessageStore,
    client: Box<dyn Client>,
    dep: Box<dyn Dep>,
) -> Listener {
    Listener {
        message_store: Rc::new(RefCell::new(message_store)),
        client,
        dep,
        handlers: Box::new(HashMap::new()),
    }
}
