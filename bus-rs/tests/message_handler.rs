#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use bus_rs::{listener::create_listener, message_handler::MessageHandler, Dep};

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

    #[test]
    fn should_register_properly_message_handler() {
        // given
        let mut listener = create_listener(Box::new(Dependencies {}));

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
        let logger = Rc::new(RefCell::new(TestLogger::new()));
        let mut listener = create_listener(Box::new(Dependencies {}));

        listener.register_handler(WrongTestMessageHandler {
            logger: logger.clone(),
        });
        listener.register_handler(TestMessageHandler {
            logger: logger.clone(),
        });

        // when
        listener.handle(TestMessage {
            data: "test_data".to_string(),
        });

        // then
        assert_eq!(2, listener.registered_handlers_count());
        assert_eq!(1, logger.borrow().get().len());
        assert_eq!("test test_data", logger.borrow().get()[0]);
    }
}
