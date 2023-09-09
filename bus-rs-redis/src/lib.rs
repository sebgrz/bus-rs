struct RedisClient;

impl RedisClient {
    pub fn new() -> RedisClient {
        RedisClient {}
    }
}

impl bus_rs::Client for RedisClient {
    fn receiver(&self, recv_callback: &dyn Fn(bus_rs::RawMessage)) {
        todo!()
    }
}
