use crate::MessageConstraints;

pub trait MessageHandler<TMessage>
where
    TMessage: MessageConstraints,
{
    fn handle(&mut self, msg: TMessage);
}

pub trait MessageHandlerRegistration {
    fn registration_name(&self) -> String;
}
