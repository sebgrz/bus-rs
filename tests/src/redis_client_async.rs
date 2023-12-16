#[cfg(test)]
mod tests {
    use bus_rs::{
        builder::{self, Builder},
        listener_async::ListenerAsync,
        publisher_async::PublisherAsync,
        ClientError, RawMessage,
    };
    use bus_rs_redis::RedisClientAsync;
    use futures_util::StreamExt as _;
    use redis::Commands;
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use testcontainers::{core::WaitFor, *};
    use tokio::sync::Mutex;

    use crate::{
        EmptyTestMessage, EmptyTestMessageHandlerAsync, SecondTestLayer, TestLayer, TestLogger,
        TestMessage, TestMessageHandlerAsync, WrongTestMessageHandlerAsync,
    };

    #[tokio::test]
    async fn should_redis_client_async_receive_message_correctly() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        let redis_client =
            RedisClientAsync::new_receiver(url.as_ref(), "test_channel".to_string()).await;
        let client = Box::new(redis_client);
        let logger = Arc::new(Mutex::new(TestLogger::new()));
        let mut listener: ListenerAsync = builder::pubsub_async(client).build();

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

        let mut test_raw_msg: bus_rs::RawMessage = TestMessage {
            data: "test_data".to_string(),
        }
        .into();
        test_raw_msg.headers = HashMap::from([("trace-id".to_owned(), "123".to_owned())]);

        // when
        tokio::spawn(async move {
            listener.listen().await.unwrap_or_else(|e| {
                if let ClientError::General(err) = e {
                    panic!("client_error: {}", err);
                }
            });
        });

        tokio::time::sleep(Duration::from_millis(200)).await;
        let test_raw_msg: String = test_raw_msg.into();
        let _: redis::Value = con.publish("test_channel", test_raw_msg).unwrap();

        // then
        tokio::time::sleep(Duration::from_millis(200)).await;

        let logger = logger.lock().await;
        assert_eq!(1, logger.get().len());
        assert_eq!("msg: test_data headers: trace-id=123", logger.get()[0]);
    }

    #[tokio::test]
    async fn should_listener_async_registered_layers_call_before_and_after_actions_in_correct_order() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        // given layer
        let logger = Arc::new(std::sync::Mutex::new(TestLogger::new()));

        let first_layer = TestLayer {
            logger: logger.clone(),
        };
        let second_layer = SecondTestLayer {
            logger: logger.clone(),
        };

        // given listener
        let redis_client =
            RedisClientAsync::new_receiver(url.as_ref(), "test_channel".to_string()).await;
        let client = Box::new(redis_client);
        let mut listener: ListenerAsync = builder::pubsub_async(client)
            .add_layer(Box::new(first_layer))
            .add_layer(Box::new(second_layer))
            .build();

        listener
            .register_handler(EmptyTestMessageHandlerAsync {})
            .await;

        let mut test_raw_msg: bus_rs::RawMessage = EmptyTestMessage {
            data: "test_data".to_string(),
        }
        .into();
        test_raw_msg.headers = HashMap::from([("trace-id".to_owned(), "123".to_owned())]);

        // when
        tokio::spawn(async move {
            listener.listen().await.unwrap_or_else(|e| {
                if let ClientError::General(err) = e {
                    panic!("client_error: {}", err);
                }
            });
        });

        tokio::time::sleep(Duration::from_millis(200)).await;
        let test_raw_msg_str: String = test_raw_msg.clone().into();
        let _: redis::Value = con.publish("test_channel", test_raw_msg_str).unwrap();

        // then
        tokio::time::sleep(Duration::from_millis(200)).await;

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

    #[tokio::test]
    async fn should_publisher_with_redis_client_async_send_message_correctly() {
        // given test receiver
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let connection = client.clone().get_async_connection().await.unwrap();

        // given publisher
        let redis_client =
            RedisClientAsync::new_sender(url.as_ref(), "test_channel".to_string()).await;
        let client = Box::new(redis_client);
        let publisher: PublisherAsync = builder::pubsub_async(client).build();

        let raw_message: Arc<Mutex<Option<bus_rs::RawMessage>>> = Arc::new(Mutex::new(None));
        let caught_raw_message = raw_message.clone();
        tokio::spawn(async move {
            let mut pubsub = connection.into_pubsub();
            pubsub.subscribe("test_channel").await.unwrap();
            let mut pubsub_stream = pubsub.on_message();

            loop {
                let msg = pubsub_stream.next().await.unwrap();
                let raw_message = bus_rs::RawMessage::from(msg.get_payload::<String>().unwrap());
                *caught_raw_message.lock().await = Some(raw_message);
            }
        });
        tokio::time::sleep(Duration::from_millis(200)).await;

        let test_msg = TestMessage {
            data: "test_data".to_string(),
        };

        // when
        let headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
        publisher.publish(&test_msg, Some(headers.clone())).await;

        // then
        tokio::time::sleep(Duration::from_millis(200)).await;

        let message_result = raw_message.lock().await.clone().unwrap();
        assert_eq!("TestMessage", message_result.msg_type);
        assert_eq!(r#"{"data":"test_data"}"#, message_result.payload);

        // and headers
        assert_eq!(headers, message_result.headers);
    }

    #[tokio::test]
    async fn should_publisher_with_registered_layers_call_before_and_after_actions_in_correct_order(
    ) {
        // given test receiver
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let connection = client.clone().get_async_connection().await.unwrap();

        // given layer
        let logger = Arc::new(std::sync::Mutex::new(TestLogger::new()));

        let first_layer = TestLayer {
            logger: logger.clone(),
        };
        let second_layer = SecondTestLayer {
            logger: logger.clone(),
        };

        // given publisher
        let redis_client =
            RedisClientAsync::new_sender(url.as_ref(), "test_channel".to_string()).await;
        let client = Box::new(redis_client);
        let publisher: PublisherAsync = builder::pubsub_async(client)
            .add_layer(Box::new(first_layer))
            .add_layer(Box::new(second_layer))
            .build();

        tokio::spawn(async move {
            let mut pubsub = connection.into_pubsub();
            pubsub.subscribe("test_channel").await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
        let test_msg = TestMessage {
            data: "test_data".to_string(),
        };
        let mut expected_msg: RawMessage = test_msg.clone().into();
        expected_msg.headers = headers.clone();

        // when
        publisher.publish(&test_msg, Some(headers)).await;

        // then
        tokio::time::sleep(Duration::from_millis(200)).await;

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
