use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use tokio::sync::Mutex;

use crate::{
    message_handler_async::MessageHandlerAsync, message_store::MessageStore, ClientAsync,
    ClientCallbackFnAsync, ClientError, MessageConstraints, PubSubLayer, RawMessage,
};

type MessageHandlerCallbackFnAsync =
    dyn Fn(&MessageStore, RawMessage) -> BoxFuture<'static, ()> + Send + Sync;

pub(super) struct ContextContainer {
    message_store: Box<MessageStore>,
    handlers: Box<HashMap<String, Arc<MessageHandlerCallbackFnAsync>>>,
    layers: Box<Vec<Box<dyn PubSubLayer>>>,
}

pub struct ListenerAsync {
    context: Arc<Mutex<ContextContainer>>,
    client: Box<dyn ClientAsync + Send + Sync + 'static>,
}

impl ListenerAsync {
    pub fn new(
        client: Box<dyn ClientAsync + Send + Sync>,
        layers: Vec<Box<dyn PubSubLayer>>,
    ) -> Self {
        let context_container = ContextContainer {
            message_store: Box::new(MessageStore::new()),
            handlers: Box::new(HashMap::new()),
            layers: Box::new(layers),
        };
        ListenerAsync {
            context: Arc::new(Mutex::new(context_container)),
            client,
        }
    }

    pub async fn listen(&mut self) -> Result<(), ClientError> {
        let context = self.context.clone();
        let callback: Arc<ClientCallbackFnAsync> = Arc::new(move |msg: RawMessage| {
            let mut msg = msg.clone();
            let context = context.clone();
            Box::pin(async move {
                let context = context.lock().await;
                context.layers.iter().for_each(|l| {
                    l.before(&mut msg);
                });

                if let Some(handler) = context.handlers.get(msg.msg_type.as_str()) {
                    handler(&context.message_store, msg.clone()).await;
                }

                context.layers.iter().rev().for_each(|l| {
                    l.after(&msg);
                });
                Ok(())
            })
        });

        self.client.receiver(callback).await
    }

    pub async fn register_handler<TMessage>(
        &mut self,
        handler: impl MessageHandlerAsync<TMessage> + Send + Sync + 'static,
    ) where
        TMessage: MessageConstraints + Send + Sync,
    {
        let handler_ref = Arc::new(Mutex::new(handler));
        let handler_fn: Box<MessageHandlerCallbackFnAsync> =
            Box::new(move |ms: &MessageStore, data: RawMessage| {
                let headers = match data.headers.is_empty() {
                    true => None,
                    false => Some(data.headers.clone()),
                };
                let msg = ms.resolve::<TMessage>(&data);
                let handler_ref = handler_ref.clone();
                Box::pin(async move {
                    let handler_ref = handler_ref.clone();
                    handler_ref.lock().await.handle(msg, headers).await;
                })
            });

        self.register_handler_callback::<TMessage>(handler_fn).await;

        let mut context = self.context.lock().await;
        context.message_store.register::<TMessage>(TMessage::name());
    }

    pub async fn registered_handlers_count(&self) -> usize {
        let context = self.context.lock().await;
        context.handlers.len()
    }

    async fn register_handler_callback<TMessage>(
        &mut self,
        callback: Box<MessageHandlerCallbackFnAsync>,
    ) where
        TMessage: MessageConstraints,
    {
        let callback: Arc<MessageHandlerCallbackFnAsync> =
            Arc::new(move |ms, msg| callback(ms, msg));

        let mut context = self.context.lock().await;
        context
            .handlers
            .insert(TMessage::name().to_string(), callback);
    }
}
