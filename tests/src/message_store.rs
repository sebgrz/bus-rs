#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bus_rs::message_store::MessageStore;
    use bus_rs::RawMessage;

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

        let raw_msg_create_user_a = RawMessage {
            msg_type: "create_user".to_string(),
            headers: HashMap::new(),
            payload: r#"{ "name": "seba" }"#.to_string(),
        };
        let raw_msg_create_user_b = RawMessage {
            msg_type: "create_user".to_string(),
            headers: HashMap::new(),
            payload: r#"{ "name": "john" }"#.to_string(),
        };
        let raw_msg_remove_user = RawMessage {
            msg_type: "remove_user".to_string(),
            headers: HashMap::new(),
            payload: r#"{ "id": 123 }"#.to_string(),
        };
        // when
        let create_user_a = store.resolve::<CreateUserMessage>(&raw_msg_create_user_a);
        let remove_user = store.resolve::<RemoveUserMessage>(&raw_msg_remove_user);
        let create_user_b = store.resolve::<CreateUserMessage>(&raw_msg_create_user_b);

        // then
        assert_eq!(123, remove_user.id);
        assert_eq!("seba", create_user_a.name);
        assert_eq!("john", create_user_b.name);
    }
}
