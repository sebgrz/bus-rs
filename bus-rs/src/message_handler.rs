use std::collections::HashMap;

use crate::MessageConstraints;

pub trait MessageHandler<TMessage>
where
    TMessage: MessageConstraints,
{
    fn handle(&mut self, msg: TMessage, headers: Option<HashMap<String, String>>);
}
