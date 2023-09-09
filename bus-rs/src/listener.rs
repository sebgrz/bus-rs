use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    message_handler::MessageHandler, message_store::MessageStore, Client, Dep, MessageConstraints,
    RawMessage,
};

pub struct Listener {
    message_store: Rc<RefCell<MessageStore>>,
    client: Rc<RefCell<dyn Client>>,
    dep: Box<dyn Dep>,
    handlers: Box<HashMap<String, Box<dyn Fn(&MessageStore, RawMessage)>>>,
}

impl Listener {
    pub fn new(client: Rc<RefCell<dyn Client>>, dep: Box<dyn Dep>) -> Self {
        Listener {
            message_store: Rc::new(RefCell::new(MessageStore::new())),
            client,
            dep,
            handlers: Box::new(HashMap::new()),
        }
    }

    pub fn listen(&mut self) {
        let callback = |msg: RawMessage| {
            self.handle(msg);
        };
        self.client.borrow().receiver(&callback);
    }

    pub fn register_handler<TMessage>(&mut self, handler: impl MessageHandler<TMessage> + 'static)
    where
        TMessage: MessageConstraints,
    {
        let handler_ref = RefCell::new(handler);
        let handler_fn = move |ms: &MessageStore, data: RawMessage| {
            let msg = ms.resolve::<TMessage>(data);
            handler_ref.borrow_mut().handle(msg);
        };

        self.message_store
            .borrow_mut()
            .register::<TMessage>(TMessage::name());
        self.register_handler_callback::<TMessage, _>(handler_fn);
    }

    pub fn registered_handlers_count(&self) -> usize {
        self.handlers.len()
    }

    fn handle(&self, msg: RawMessage) {
        if let Some(handler) = self.handlers.get(msg.msg_type.as_str()) {
            handler(&self.message_store.borrow(), msg);
        }
    }

    fn register_handler_callback<TMessage, TCallback>(&mut self, callback: TCallback)
    where
        TMessage: MessageConstraints,
        TCallback: Fn(&MessageStore, RawMessage) + 'static,
    {
        let callback: Box<dyn Fn(&MessageStore, RawMessage)> =
            Box::new(move |ms, msg| callback(ms, msg));

        self.handlers.insert(TMessage::name().to_string(), callback);
    }
}
