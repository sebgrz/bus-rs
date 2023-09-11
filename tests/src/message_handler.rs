#[cfg(test)]
mod tests {
    use bus_rs::{listener::Listener, Client, RawMessage};

    use std::{cell::RefCell, rc::Rc, sync::{Mutex, Arc}};

    use crate::{Dependencies, TestLogger, TestMessageHandler, WrongTestMessageHandler};

    #[test]
    fn should_register_properly_message_handler() {
        // given
        let client = Arc::new(Mutex::new(MockClient::new()));
        let dep = Box::new(Dependencies {});
        let mut listener = Listener::new(client, dep);

        // when
        listener.register_handler(TestMessageHandler {
            logger: Arc::new(Mutex::new(TestLogger::new())),
        });

        // then
        assert_eq!(1, listener.registered_handlers_count());
    }

    #[test]
    fn should_message_invoke_msg_handler_correctly() {
        // given
        let client = Arc::new(Mutex::new(MockClient::new()));
        let dep = Box::new(Dependencies {});
        let logger = Arc::new(Mutex::new(TestLogger::new()));
        let mut listener = Listener::new(client.clone(), dep);

        listener.register_handler(WrongTestMessageHandler {
            logger: logger.clone(),
        });
        listener.register_handler(TestMessageHandler {
            logger: logger.clone(),
        });

        client.lock().unwrap().push_message(RawMessage {
            msg_type: "TestMessage".to_string(),
            payload: r#"{ "data": "test_data" }"#.to_string(),
        });

        // when
        listener.listen();

        // then
        let logger = logger.lock().unwrap();
        assert_eq!(2, listener.registered_handlers_count());
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

    impl Client for MockClient {
        fn receiver(&mut self, recv_callback: &dyn Fn(bus_rs::RawMessage)) {
            for msg in self.messages.iter() {
                recv_callback(msg.clone());
            }
        }
    }
}
