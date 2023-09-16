use async_trait::async_trait;

use crate::MessageConstraints;

#[async_trait]
pub trait MessageHandlerAsync<TMessage>
where
    TMessage: MessageConstraints,
{
    async fn handle(&mut self, msg: TMessage);
}
