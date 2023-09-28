use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bus_rs::{message_handler::MessageHandler, message_handler_async::MessageHandlerAsync, Dep};
use bus_rs_macros::message;
use serde::{Deserialize, Serialize};

mod message_handler;
mod message_handler_async;
mod message_store;
mod redis_client;
mod redis_client_async;

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

struct Dependencies;

impl Dep for Dependencies {}

// message handlers
#[message]
#[derive(Deserialize, Serialize)]
struct TestMessage {
    data: String,
}

struct TestMessageHandler {
    logger: Arc<Mutex<TestLogger>>,
}

impl MessageHandler<TestMessage> for TestMessageHandler {
    fn handle(&mut self, msg: TestMessage) {
        let mut l = self.logger.lock().unwrap();
        l.info(format!("test {}", msg.data));
    }
}

#[message]
#[derive(Deserialize, Serialize)]
struct WrongTestMessage {
    data: String,
}

struct WrongTestMessageHandler {
    logger: Arc<Mutex<TestLogger>>,
}

impl MessageHandler<WrongTestMessage> for WrongTestMessageHandler {
    fn handle(&mut self, msg: WrongTestMessage) {
        let mut l = self.logger.lock().unwrap();
        l.info(format!("wrong test {}", msg.data));
    }
}

// async
struct WrongTestMessageHandlerAsync {
    logger: Arc<tokio::sync::Mutex<TestLogger>>,
}

#[async_trait]
impl MessageHandlerAsync<WrongTestMessage> for WrongTestMessageHandlerAsync {
    async fn handle(&mut self, msg: WrongTestMessage) {
        let mut l = self.logger.lock().await;
        l.info(format!("wrong test {}", msg.data));
    }
}

struct TestMessageHandlerAsync {
    logger: Arc<tokio::sync::Mutex<TestLogger>>,
}

#[async_trait]
impl MessageHandlerAsync<TestMessage> for TestMessageHandlerAsync {
    async fn handle(&mut self, msg: TestMessage) {
        let mut l = self.logger.lock().await;
        l.info(format!("test {}", msg.data));
    }
}
