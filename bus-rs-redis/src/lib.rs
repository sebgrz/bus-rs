struct RedisClient {
    connection: Box<redis::Connection>,
    channel: &'static str,
}

impl RedisClient {
    pub fn new_receiver(addr: &str, channel: &'static str) -> RedisClient {
        let redis_client = redis::Client::open(addr).unwrap();
        let conn = redis_client.get_connection().unwrap();
        RedisClient {
            connection: Box::new(conn),
            channel,
        }
    }
}

impl bus_rs::Client for RedisClient {
    fn receiver(&mut self, recv_callback: &dyn Fn(bus_rs::RawMessage)) {
        let mut pubsub = self.connection.as_pubsub();
        pubsub.subscribe(self.channel).unwrap();
        loop {
            let msg = pubsub.get_message().unwrap();
            let raw_message = bus_rs::RawMessage::from(msg.get_payload::<String>().unwrap());
            recv_callback(raw_message);
        }
    }
}
