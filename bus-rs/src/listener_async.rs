use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use tokio::sync::Mutex;

use crate::{
    message_handler_async::MessageHandlerAsync, message_store::MessageStore, ClientAsync,
    ClientCallbackFnAsync, ClientError, MessageConstraints, RawMessage,
};

type MessageHandlerCallbackFnAsync =
    dyn Fn(Arc<Mutex<MessageStore>>, RawMessage) -> BoxFuture<'static, ()> + Send + Sync;

pub struct ListenerAsync {
    message_store: Arc<Mutex<MessageStore>>,
    client: Arc<Mutex<dyn ClientAsync + Send + Sync + 'static>>,
    handlers: Arc<Mutex<HashMap<String, Arc<MessageHandlerCallbackFnAsync>>>>,
}

impl ListenerAsync {
    pub fn new(client: Arc<Mutex<dyn ClientAsync + Send + Sync>>) -> Self {
        ListenerAsync {
            message_store: Arc::new(Mutex::new(MessageStore::new())),
            client,
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn listen(&mut self) -> Result<(), ClientError> {
        let handlers = self.handlers.clone();
        let message_store = self.message_store.clone();
        let callback: Arc<ClientCallbackFnAsync> = Arc::new(move |msg: RawMessage| {
            let handlers = handlers.clone();
            let message_store = message_store.clone();
            Box::pin(async move {
                if let Some(handler) = handlers.lock().await.get(msg.msg_type.as_str()) {
                    handler(message_store, msg).await;
                }
                Ok(())
            })
        });

        self.client.lock().await.receiver(callback).await
    }

    pub async fn register_handler<TMessage>(
        &mut self,
        handler: impl MessageHandlerAsync<TMessage> + Send + Sync + 'static,
    ) where
        TMessage: MessageConstraints + Send + Sync,
    {
        let handler_ref = Arc::new(Mutex::new(handler));
        let handler_fn: Box<MessageHandlerCallbackFnAsync> =
            Box::new(move |ms: Arc<Mutex<MessageStore>>, data: RawMessage| {
                let handler_ref = handler_ref.clone();
                Box::pin(async move {
                    let handler_ref = handler_ref.clone();
                    let msg = ms.lock().await.resolve::<TMessage>(data);
                    handler_ref.lock().await.handle(msg).await;
                })
            });

        self.message_store
            .lock()
            .await
            .register::<TMessage>(TMessage::name());

        self.register_handler_callback::<TMessage>(handler_fn).await;
    }

    pub async fn registered_handlers_count(&self) -> usize {
        self.handlers.lock().await.len()
    }

    async fn register_handler_callback<TMessage>(
        &mut self,
        callback: Box<MessageHandlerCallbackFnAsync>,
    ) where
        TMessage: MessageConstraints,
    {
        let callback: Arc<MessageHandlerCallbackFnAsync> =
            Arc::new(move |ms, msg| callback(ms, msg));

        self.handlers
            .lock()
            .await
            .insert(TMessage::name().to_string(), callback);
    }
}
