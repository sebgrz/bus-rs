use serde::de::DeserializeOwned;
use std::{any::Any, collections::HashMap};

use crate::Message;

pub struct MessageStore {
    messages: Box<HashMap<String, Box<dyn Fn(Box<&str>) -> Box<dyn Any>>>>,
}

impl MessageStore {
    pub fn new() -> Self {
        MessageStore {
            messages: Box::new(HashMap::new()),
        }
    }

    pub fn register<TMessage>(&mut self, key: &str)
    where
        TMessage: DeserializeOwned + 'static,
    {
        let callback: Box<dyn Fn(Box<&str>) -> Box<dyn Any>> = Box::new(|msg_payload| {
            let msg: TMessage = serde_json::from_str(&msg_payload).unwrap();
            let msg: Box<dyn Any> = Box::new(msg);
            return msg;
        });

        self.messages.insert(key.to_string(), callback);
    }

    pub fn resolve<TMessage>(&self, raw_message: Message) -> TMessage
    where
        TMessage: DeserializeOwned + 'static,
    {
        let msg_fn = self.messages.get(&raw_message.msg_type).unwrap();
        let msg = msg_fn(Box::new(&raw_message.payload));
        let msg: Box<TMessage> = msg.downcast::<TMessage>().unwrap();
        return *msg;
    }
}

#[cfg(test)]
mod tests {
    use crate::Message;

    use super::MessageStore;

    #[derive(serde::Deserialize)]
    struct CreateUserMessage {
        name: String,
    }

    #[derive(serde::Deserialize)]
    struct RemoveUserMessage {
        id: i32,
    }

    #[test]
    fn should_register_and_resolve_message_correctly() {
        // given
        let mut store = MessageStore::new();

        store.register::<CreateUserMessage>("create_user");
        store.register::<RemoveUserMessage>("remove_user");

        let raw_msg_create_user_a = Message {
            msg_type: "create_user".to_string(),
            payload: r#"{ "name": "seba" }"#.to_string(),
        };
        let raw_msg_create_user_b = Message {
            msg_type: "create_user".to_string(),
            payload: r#"{ "name": "john" }"#.to_string(),
        };
        let raw_msg_remove_user = Message {
            msg_type: "remove_user".to_string(),
            payload: r#"{ "id": 123 }"#.to_string(),
        };
        // when
        let create_user_a = store.resolve::<CreateUserMessage>(raw_msg_create_user_a);
        let remove_user = store.resolve::<RemoveUserMessage>(raw_msg_remove_user);
        let create_user_b = store.resolve::<CreateUserMessage>(raw_msg_create_user_b);

        // then
        assert_eq!(123, remove_user.id);
        assert_eq!("seba", create_user_a.name);
        assert_eq!("john", create_user_b.name);
    }
}
