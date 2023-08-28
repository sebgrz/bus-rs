#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use bus_rs::{message_handler::MessageHandler, listener::create_listener, Dep};

    struct TestMessage {
        data: String
    }

    struct TestMessageHandler;

    impl MessageHandler<TestMessage> for TestMessageHandler {
        fn handle(&mut self, msg: TestMessage) {
            println!("test {}", msg.data);
        }
    }

    struct Dependencies;

    impl Dep for Dependencies {}

    #[test]
    fn listener_registration_handler() {
        // given
        let mut listener = create_listener(Box::new(Dependencies{}));

        // when
        listener.register_handler(TestMessageHandler{});

        // then
        assert_eq!(1, listener.registered_handlers_count());
    }
}
