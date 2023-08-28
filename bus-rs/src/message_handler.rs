use std::rc::Rc;

use crate::Message;

pub trait MessageHandler<TMessage>
where
    TMessage: 'static,
{
    fn handle(&mut self, msg: TMessage);
}

pub trait MessageHandlerRegistration {
    fn registration_name(&self) -> String;
}

pub fn message_handler_dispatcher(msg: Message) {}
