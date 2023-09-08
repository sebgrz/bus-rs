use crate::MessageConstraints;

pub trait MessageHandler<TMessage>
where
    TMessage: MessageConstraints,
{
    fn handle(&mut self, msg: TMessage);
}
