#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use bus_rs::{listener_async::ListenerAsync, ClientError};
    use bus_rs_redis::RedisClientAsync;
    use redis::Commands;
    use testcontainers::{core::WaitFor, *};
    use tokio::sync::Mutex;

    use crate::{
        Dependencies, TestLogger, TestMessage, TestMessageHandlerAsync,
        WrongTestMessageHandlerAsync,
    };

    #[tokio::test]
    async fn should_redis_client_async_receive_message_correctly() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        let redis_client = RedisClientAsync::new_receiver(url.as_ref(), "test_channel").await;
        let client = Arc::new(Mutex::new(redis_client));
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

        let test_raw_msg: bus_rs::RawMessage = TestMessage {
            data: "test_data".to_string(),
        }
        .into();

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
        assert_eq!("test test_data", logger.get()[0]);
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
