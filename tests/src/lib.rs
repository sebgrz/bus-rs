use std::{cell::RefCell, rc::Rc, sync::{Mutex, Arc}};

use bus_rs::{message_handler::MessageHandler, Dep};
use bus_rs_macros::message;
use serde::{Deserialize, Serialize};

mod message_handler;
mod message_store;

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

struct Dependencies;

impl Dep for Dependencies {}
