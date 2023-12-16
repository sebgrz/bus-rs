use bus_rs::ClientError;
use redis::Commands;

pub struct RedisClient {
    connection: Box<redis::Connection>,
    channel: String,
}

impl RedisClient {
    pub fn new(addr: &str, channel: String) -> RedisClient {
        let redis_client = redis::Client::open(addr).unwrap();
        let conn = redis_client.get_connection().unwrap();
        RedisClient {
            connection: Box::new(conn),
            channel,
        }
    }
}

impl bus_rs::Client for RedisClient {
    fn receiver(&mut self, recv_callback: &dyn Fn(bus_rs::RawMessage)) -> Result<(), ClientError> {
        let mut pubsub = self.connection.as_pubsub();
        pubsub.subscribe(self.channel.as_str()).unwrap();

        loop {
            let msg = pubsub.get_message().map(|m| Ok(m)).unwrap_or_else(|e| {
                if e.is_io_error() {
                    return Err(ClientError::IO(e.to_string()));
                }
                return Err(ClientError::General(e.to_string()));
            })?;
            let raw_message = bus_rs::RawMessage::from(msg.get_payload::<String>().unwrap());
            recv_callback(raw_message);
        }
    }

    fn send(&mut self, msg: &bus_rs::RawMessage) -> Result<(), ClientError> {
        let str_msg: String = msg.into();
        let result: Result<(), redis::RedisError> =
            self.connection.publish(self.channel.as_str(), str_msg);

        let _ = result.or_else(|e| {
            if e.is_io_error() {
                return Err(ClientError::IO(e.to_string()));
            }
            return Err(ClientError::General(e.to_string()));
        });

        Ok(())
    }
}
