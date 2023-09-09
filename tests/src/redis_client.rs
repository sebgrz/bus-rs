#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        rc::Rc,
        thread::{sleep, spawn},
        time::Duration, sync::{Arc, Mutex},
    };

    use bus_rs::{listener::Listener, Client};
    use bus_rs_redis::RedisClient;
    use redis::Commands;
    use testcontainers::{core::WaitFor, *};

    use crate::{Dependencies, TestLogger, TestMessageHandler, WrongTestMessageHandler, TestMessage};

    #[test]
    fn should_redis_client_receive_message_correctly() {
        // given
        let docker_client = clients::Cli::default();
        let (_node, url) = prepare_redis_container(&docker_client);
        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();

        let redis_client = RedisClient::new_receiver(url.as_ref(), "test_channel");
        let client = Arc::new(Mutex::new(redis_client));
        let dep = Box::new(Dependencies {});
        let logger = Arc::new(Mutex::new(TestLogger::new()));
        let mut listener = Listener::new(client.clone(), dep);

        listener.register_handler(WrongTestMessageHandler {
            logger: logger.clone(),
        });
        listener.register_handler(TestMessageHandler {
            logger: logger.clone(),
        });

        let test_raw_msg: bus_rs::RawMessage = TestMessage {
            data: "test_data".to_string()
        }.into();

        // when
        spawn(move || {
            listener.listen();
        });

        sleep(Duration::from_millis(200));
        let test_raw_msg:String = test_raw_msg.into();
        let _: redis::Value = con.publish("test_channel", test_raw_msg).unwrap();

        // then
        sleep(Duration::from_millis(200));

        let logger = logger.lock().unwrap();
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
