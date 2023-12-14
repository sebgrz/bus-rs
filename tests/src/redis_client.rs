#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread::{sleep, spawn},
        time::Duration,
    };

    use bus_rs::{listener::Listener, publisher::Publisher, ClientError};
    use bus_rs_redis::RedisClient;
    use redis::Commands;
    use testcontainers::{core::WaitFor, *};

    use crate::{TestLogger, TestMessage, TestMessageHandler, WrongTestMessageHandler};

    #[test]
    fn should_listener_with_redis_client_receive_message_correctly() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        let redis_client = RedisClient::new(url.as_ref(), "test_channel".to_string());
        let client = Arc::new(Mutex::new(redis_client));
        let logger = Arc::new(Mutex::new(TestLogger::new()));
        let mut listener = Listener::new(client.clone());

        listener.register_handler(WrongTestMessageHandler {
            logger: logger.clone(),
        });
        listener.register_handler(TestMessageHandler {
            logger: logger.clone(),
        });

        let test_raw_msg: bus_rs::RawMessage = TestMessage {
            data: "test_data".to_string(),
        }
        .into();

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
        assert_eq!("test test_data", logger.get()[0]);
    }

    #[test]
    fn should_publiher_with_redis_client_send_message_correctly() {
        // given test receiver
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut connection = client.clone().get_connection().unwrap();

        // given publisher
        let redis_client = RedisClient::new(url.as_ref(), "test_channel".to_string());
        let client = Arc::new(Mutex::new(redis_client));
        let publisher = Publisher::new(client.clone());

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
        publisher.publish(&test_msg);

        // then
        sleep(Duration::from_millis(200));

        let message_result = raw_message.lock().unwrap().clone().unwrap();
        assert_eq!("TestMessage", message_result.msg_type);
        assert_eq!(r#"{"data":"test_data"}"#, message_result.payload);
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
