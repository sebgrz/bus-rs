pub trait MessageHandler {
    type TMessage;

    fn handle(&mut self, msg: Self::TMessage);
}
