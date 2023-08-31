use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    message_handler::{message_handler_dispatcher, MessageHandler},
    Dep, Message,
};

pub struct Listener {
    dep: Box<dyn Dep>,
    handlers: Box<HashMap<String, Box<dyn Fn(&mut dyn Any)>>>,
}

impl<'a> Iterator for Listener {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Listener {
    pub fn listen(&mut self) {
        for message in self.into_iter() {
            message_handler_dispatcher(message);
        }
    }

    pub fn register_handler<TMessage>(
        &mut self,
        handler: impl MessageHandler<TMessage> + 'static,
    ) where
        TMessage: 'static,
    {
        let a = RefCell::new(handler);
        let m = move |data: TMessage| {
            a.borrow_mut().handle(data);
        };
        self.register_handler_callback(m);
    }

    pub fn handle<TMessage>(&self, msg: TMessage)
        where TMessage: 'static
    {
        let message_name = std::any::type_name::<TMessage>();
        if let Some(handler) = self.handlers.get(message_name) {
            handler(&mut Some(msg));
        }
    }

    pub fn registered_handlers_count(&self) -> usize {
        self.handlers.len()
    }

    fn register_handler_callback<TMessage, TCallback>(&mut self, callback: TCallback)
    where
        TMessage: 'static,
        TCallback: Fn(TMessage) + 'static,
    {
        let message_name = std::any::type_name::<TMessage>();
        let callback: Box<dyn Fn(&mut dyn Any)> = Box::new(move |msg| {
            let msg = msg.downcast_mut::<Option<TMessage>>().unwrap();
            callback(msg.take().unwrap())
        });

        self.handlers.insert(message_name.to_string(), callback);
    }
}

pub fn create_listener(/* TODO: bus type as parameter */ dep: Box<dyn Dep>) -> Listener {
    Listener {
        dep,
        handlers: Box::new(HashMap::new()),
    }
}
