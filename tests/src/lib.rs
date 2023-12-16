use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bus_rs::{
    message_handler::MessageHandler, message_handler_async::MessageHandlerAsync, PubSubLayer,
};
use bus_rs_macros::message;
use itertools::Itertools;
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

// pubsub layer
struct TestLayer {
    logger: Arc<Mutex<TestLogger>>,
}

impl PubSubLayer for TestLayer {
    fn before(&self, raw_msg: &mut bus_rs::RawMessage) {
        self.logger
            .lock()
            .unwrap()
            .info(format!("TestLayer before | msg: {:?}", raw_msg));
    }

    fn after(&self, raw_msg: &bus_rs::RawMessage) {
        self.logger
            .lock()
            .unwrap()
            .info(format!("TestLayer after | msg: {:?}", raw_msg));
    }
}

struct SecondTestLayer {
    logger: Arc<Mutex<TestLogger>>,
}

impl PubSubLayer for SecondTestLayer {
    fn before(&self, raw_msg: &mut bus_rs::RawMessage) {
        self.logger
            .lock()
            .unwrap()
            .info(format!("SecondTestLayer before | msg: {:?}", raw_msg));
    }

    fn after(&self, raw_msg: &bus_rs::RawMessage) {
        self.logger
            .lock()
            .unwrap()
            .info(format!("SecondTestLayer after | msg: {:?}", raw_msg));
    }
}

// message handlers
#[message]
#[derive(Deserialize, Serialize, Clone)]
struct EmptyTestMessage {
    data: String,
}

struct EmptyTestMessageHandler;

impl MessageHandler<EmptyTestMessage> for EmptyTestMessageHandler {
    fn handle(&mut self, _msg: EmptyTestMessage, _headers: Option<HashMap<String, String>>) {}
}

#[message]
#[derive(Deserialize, Serialize, Clone)]
struct TestMessage {
    data: String,
}

struct TestMessageHandler {
    logger: Arc<Mutex<TestLogger>>,
}

impl MessageHandler<TestMessage> for TestMessageHandler {
    fn handle(&mut self, msg: TestMessage, headers: Option<HashMap<String, String>>) {
        let mut l = self.logger.lock().unwrap();
        let headers_str: String = match headers {
            Some(h) => h.iter().map(|(k, v)| format!("{}={}", k, v)).join(","),
            None => "".to_string(),
        };
        l.info(format!("msg: {} headers: {}", msg.data, headers_str));
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
    fn handle(&mut self, msg: WrongTestMessage, _headers: Option<HashMap<String, String>>) {
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
    async fn handle(&mut self, msg: WrongTestMessage, _headers: Option<HashMap<String, String>>) {
        let mut l = self.logger.lock().await;
        l.info(format!("wrong test {}", msg.data));
    }
}

struct TestMessageHandlerAsync {
    logger: Arc<tokio::sync::Mutex<TestLogger>>,
}

#[async_trait]
impl MessageHandlerAsync<TestMessage> for TestMessageHandlerAsync {
    async fn handle(&mut self, msg: TestMessage, headers: Option<HashMap<String, String>>) {
        let mut l = self.logger.lock().await;
        let headers_str: String = match headers {
            Some(h) => h.iter().map(|(k, v)| format!("{}={}", k, v)).join(","),
            None => "".to_string(),
        };
        l.info(format!("msg: {} headers: {}", msg.data, headers_str));
    }
}

struct EmptyTestMessageHandlerAsync;

#[async_trait]
impl MessageHandlerAsync<EmptyTestMessage> for EmptyTestMessageHandlerAsync {
    async fn handle(&mut self, _msg: EmptyTestMessage, _headers: Option<HashMap<String, String>>) {}
}
