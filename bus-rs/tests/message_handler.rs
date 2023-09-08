#[cfg(test)]
mod tests {
    use bus_rs::{
        listener::Listener, message_handler::MessageHandler, Client, Dep,
        RawMessage,
    };
    use bus_rs_macros::MessageResolver;
    use serde::Deserialize;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn should_register_properly_message_handler() {
        // given
        let client = Rc::new(RefCell::new(MockClient::new()));
        let dep = Box::new(Dependencies {});
        let mut listener = Listener::new(client, dep);

        // when
        listener.register_handler(TestMessageHandler {
            logger: Rc::new(RefCell::new(TestLogger::new())),
        });

        // then
        assert_eq!(1, listener.registered_handlers_count());
    }

    #[test]
    fn should_message_invoke_msg_handler_correctly() {
        // given
        let client = Rc::new(RefCell::new(MockClient::new()));
        let dep = Box::new(Dependencies {});
        let logger = Rc::new(RefCell::new(TestLogger::new()));
        let mut listener = Listener::new(client.clone(), dep);

        listener.register_handler(WrongTestMessageHandler {
            logger: logger.clone(),
        });
        listener.register_handler(TestMessageHandler {
            logger: logger.clone(),
        });

        client.borrow_mut().push_message(RawMessage {
            msg_type: "TestMessage".to_string(),
            payload: r#"{ "data": "test_data" }"#.to_string(),
        });

        // when
        listener.listen();

        // then
        assert_eq!(2, listener.registered_handlers_count());
        assert_eq!(1, logger.borrow().get().len());
        assert_eq!("test test_data", logger.borrow().get()[0]);
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
        fn receiver(&self, recv_callback: &dyn Fn(bus_rs::RawMessage)) {
            for msg in self.messages.iter() {
                recv_callback(msg.clone());
            }
        }
    }

    struct TestLogger {
        messages: Vec<String>,
    }

    impl TestLogger {
        fn new() -> Self {
            TestLogger { messages: vec![] }
        }

        fn info(&mut self, msg: String) {
            self.messages.push(msg);
        }

        fn get(&self) -> &Vec<String> {
            &self.messages
        }

        fn clear(&mut self) {
            self.messages.clear();
        }
    }

    #[derive(Deserialize, MessageResolver)]
    struct TestMessage {
        data: String,
    }

    struct TestMessageHandler {
        logger: Rc<RefCell<TestLogger>>,
    }

    impl MessageHandler<TestMessage> for TestMessageHandler {
        fn handle(&mut self, msg: TestMessage) {
            let mut l = self.logger.borrow_mut();
            l.info(format!("test {}", msg.data));
        }
    }

    #[derive(Deserialize, MessageResolver)]
    struct WrongTestMessage {
        data: String,
    }

    struct WrongTestMessageHandler {
        logger: Rc<RefCell<TestLogger>>,
    }

    impl MessageHandler<WrongTestMessage> for WrongTestMessageHandler {
        fn handle(&mut self, msg: WrongTestMessage) {
            let mut l = self.logger.borrow_mut();
            l.info(format!("wrong test {}", msg.data));
        }
    }

    struct Dependencies;

    impl Dep for Dependencies {}
}
