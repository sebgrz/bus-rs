#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        thread::{sleep, spawn},
        time::Duration,
    };

    use bus_rs::{
        builder::{self, Builder},
        listener::Listener,
        publisher::Publisher,
        ClientError, RawMessage,
    };
    use bus_rs_redis::RedisClient;
    use redis::Commands;
    use testcontainers::{core::WaitFor, *};

    use crate::{
        EmptyTestMessage, EmptyTestMessageHandler, SecondTestLayer, TestLayer, TestLogger,
        TestMessage, TestMessageHandler, WrongTestMessageHandler,
    };

    #[test]
    fn should_listener_with_redis_client_receive_message_correctly() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        let redis_client = RedisClient::new(url.as_ref(), "test_channel".to_string());
        let client = Box::new(redis_client);
        let logger = Arc::new(Mutex::new(TestLogger::new()));
        let mut listener: Listener = builder::pubsub(client).build();

        listener.register_handler(WrongTestMessageHandler {
            logger: logger.clone(),
        });
        listener.register_handler(TestMessageHandler {
            logger: logger.clone(),
        });

        let mut test_raw_msg: bus_rs::RawMessage = TestMessage {
            data: "test_data".to_string(),
        }
        .into();
        test_raw_msg.headers = HashMap::from([("trace-id".to_owned(), "123".to_owned())]);

        // when
        spawn(move || {
            listener.listen().unwrap_or_else(|e| {
                if let ClientError::General(err) = e {
                    panic!("client_error: {}", err);
                }
            });
        });

        sleep(Duration::from_millis(200));
        let test_raw_msg: String = test_raw_msg.into();
        let _: redis::Value = con.publish("test_channel", test_raw_msg).unwrap();

        // then
        sleep(Duration::from_millis(200));

        let logger = logger.lock().unwrap();
        assert_eq!(1, logger.get().len());
        assert_eq!("msg: test_data headers: trace-id=123", logger.get()[0]);
    }

    #[test]
    fn should_listener_with_registered_layers_call_before_and_after_actions_in_correct_order() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        // given layer
        let logger = Arc::new(Mutex::new(TestLogger::new()));

        let first_layer = TestLayer {
            logger: logger.clone(),
        };
        let second_layer = SecondTestLayer {
            logger: logger.clone(),
        };

        // given listener
        let redis_client = RedisClient::new(url.as_ref(), "test_channel".to_string());
        let client = Box::new(redis_client);
        let mut listener: Listener = builder::pubsub(client)
            .add_layer(Box::new(first_layer))
            .add_layer(Box::new(second_layer))
            .build();

        listener.register_handler(EmptyTestMessageHandler {});

        let mut test_raw_msg: bus_rs::RawMessage = EmptyTestMessage {
            data: "test_data".to_string(),
        }
        .into();
        test_raw_msg.headers = HashMap::from([("trace-id".to_owned(), "123".to_owned())]);

        // when
        spawn(move || {
            listener.listen().unwrap_or_else(|e| {
                if let ClientError::General(err) = e {
                    panic!("client_error: {}", err);
                }
            });
        });

        sleep(Duration::from_millis(200));
        let test_raw_msg_str: String = test_raw_msg.clone().into();
        let _: redis::Value = con.publish("test_channel", test_raw_msg_str).unwrap();

        // then
        sleep(Duration::from_millis(200));

        let logger = logger.lock().unwrap();
        assert_eq!(4, logger.messages.len());
        assert_eq!(
            format!("TestLayer before | msg: {:?}", test_raw_msg),
            logger.messages[0]
        );
        assert_eq!(
            format!("SecondTestLayer before | msg: {:?}", test_raw_msg),
            logger.messages[1]
        );
        assert_eq!(
            format!("SecondTestLayer after | msg: {:?}", test_raw_msg),
            logger.messages[2]
        );
        assert_eq!(
            format!("TestLayer after | msg: {:?}", test_raw_msg),
            logger.messages[3]
        );
    }

    #[test]
    fn should_publisher_with_redis_client_send_message_correctly() {
        // given test receiver
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut connection = client.clone().get_connection().unwrap();

        // given publisher
        let redis_client = RedisClient::new(url.as_ref(), "test_channel".to_string());
        let client = Box::new(redis_client);
        let publisher: Publisher = builder::pubsub(client).build();

        let raw_message: Arc<Mutex<Option<bus_rs::RawMessage>>> = Arc::new(Mutex::new(None));
        let caught_raw_message = raw_message.clone();
        spawn(move || {
            let mut pubsub = connection.as_pubsub();
            pubsub.subscribe("test_channel").unwrap();

            loop {
                let msg = pubsub.get_message().unwrap_or_else(|e| {
                    panic!("get_message err: {:?}", e);
                });
                *caught_raw_message.lock().unwrap() = Some(bus_rs::RawMessage::from(
                    msg.get_payload::<String>().unwrap(),
                ));
            }
        });
        sleep(Duration::from_millis(200));

        let test_msg = TestMessage {
            data: "test_data".to_string(),
        };

        // when
        let headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
        publisher.publish(&test_msg, Some(headers));

        // then
        sleep(Duration::from_millis(200));

        let message_result = raw_message.lock().unwrap().clone().unwrap();
        assert_eq!("TestMessage", message_result.msg_type);
        assert_eq!(r#"{"data":"test_data"}"#, message_result.payload);

        // and headers
        let expected_headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
        assert_eq!(expected_headers, message_result.headers);
    }

    #[test]
    fn should_publisher_with_registered_layers_call_before_and_after_actions_in_correct_order() {
        // given test receiver
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut connection = client.clone().get_connection().unwrap();

        // given layer
        let logger = Arc::new(Mutex::new(TestLogger::new()));

        let first_layer = TestLayer {
            logger: logger.clone(),
        };
        let second_layer = SecondTestLayer {
            logger: logger.clone(),
        };
        // given publisher
        let redis_client = RedisClient::new(url.as_ref(), "test_channel".to_string());
        let client = Box::new(redis_client);
        let publisher: Publisher = builder::pubsub(client)
            .add_layer(Box::new(first_layer))
            .add_layer(Box::new(second_layer))
            .build();

        spawn(move || {
            let mut pubsub = connection.as_pubsub();
            pubsub.subscribe("test_channel").unwrap();
        });
        sleep(Duration::from_millis(200));

        // given messages
        let headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
        let test_msg = TestMessage {
            data: "test_data".to_string(),
        };
        let mut expected_msg: RawMessage = test_msg.clone().into();
        expected_msg.headers = headers.clone();

        // when
        publisher.publish(&test_msg, Some(headers));

        // then
        sleep(Duration::from_millis(200));

        let logger = logger.lock().unwrap();
        assert_eq!(4, logger.messages.len());
        assert_eq!(
            format!("TestLayer before | msg: {:?}", expected_msg),
            logger.messages[0]
        );
        assert_eq!(
            format!("SecondTestLayer before | msg: {:?}", expected_msg),
            logger.messages[1]
        );
        assert_eq!(
            format!("SecondTestLayer after | msg: {:?}", expected_msg),
            logger.messages[2]
        );
        assert_eq!(
            format!("TestLayer after | msg: {:?}", expected_msg),
            logger.messages[3]
        );
    }

    fn prepare_redis_container<'a>(docker: &'a clients::Cli) -> (Container<'a, Redis>, String) {
        let node = docker.run(Redis::default());
        let host_port = node.get_host_port_ipv4(6379);
        (node, format!("redis://127.0.0.1:{}", host_port))
    }

    const NAME: &str = "redis";
    const TAG: &str = "7.2.1-alpine";

    #[derive(Debug, Default)]
    pub struct Redis;

    impl Image for Redis {
        type Args = ();

        fn name(&self) -> String {
            NAME.to_owned()
        }

        fn tag(&self) -> String {
            TAG.to_owned()
        }

        fn ready_conditions(&self) -> Vec<WaitFor> {
            vec![WaitFor::message_on_stdout("Ready to accept connections")]
        }
    }
}
