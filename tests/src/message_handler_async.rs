#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use bus_rs::{
        listener_async::ListenerAsync, ClientAsync, ClientCallbackFnAsync, ClientError, RawMessage,
    };
    use tokio::sync::Mutex;

    use std::sync::Arc;

    use crate::{Dependencies, TestLogger, TestMessageHandlerAsync, WrongTestMessageHandlerAsync};

    #[tokio::test]
    async fn should_register_properly_message_handler_async() {
        // given
        let client = Arc::new(Mutex::new(MockClient::new()));
        let dep = Box::new(Dependencies {});
        let mut listener = ListenerAsync::new(client, dep);

        // when
        listener
            .register_handler(TestMessageHandlerAsync {
                logger: Arc::new(Mutex::new(TestLogger::new())),
            })
            .await;

        // then
        assert_eq!(1, listener.registered_handlers_count().await);
    }

    #[tokio::test]
    async fn should_message_invoke_msg_handler_async_correctly() {
        // given
        let client = Arc::new(Mutex::new(MockClient::new()));
        let dep = Box::new(Dependencies {});
        let logger = Arc::new(Mutex::new(TestLogger::new()));
        let mut listener = ListenerAsync::new(client.clone(), dep);

        listener
            .register_handler(WrongTestMessageHandlerAsync {
                logger: logger.clone(),
            })
            .await;
        listener
            .register_handler(TestMessageHandlerAsync {
                logger: logger.clone(),
            })
            .await;

        client.lock().await.push_message(RawMessage {
            msg_type: "TestMessage".to_string(),
            payload: r#"{ "data": "test_data" }"#.to_string(),
        });

        // when
        let _ = listener.listen().await;

        // then
        let logger = logger.lock().await;
        assert_eq!(2, listener.registered_handlers_count().await);
        assert_eq!(1, logger.get().len());
        assert_eq!("test test_data", logger.get()[0]);
    }

    // Helpers
    struct MockClient {
        messages: Vec<RawMessage>,
    }

    impl MockClient {
        fn new() -> Self {
            MockClient { messages: vec![] }
        }

        fn push_message(&mut self, msg: RawMessage) {
            self.messages.push(msg);
        }
    }

    #[async_trait]
    impl ClientAsync for MockClient {
        async fn receiver(
            &mut self,
            recv_callback: Arc<ClientCallbackFnAsync>,
        ) -> Result<(), ClientError> {
            for msg in self.messages.iter() {
                let _ = recv_callback(msg.clone()).await;
            }
            Ok(())
        }
    }
}
