# bus-rs
Universal (client expandable) message bus library with redis pub-sub implementation in rust.  
**Dev version - only for tests**

# Installation
Add below line to your Cargo.toml dependencies config:
```
[dependencies]
bus-rs = { git = "https://github.com/sebgrz/bus-rs.git", tag = "v0.3.0" }
```

# Message Handler implementation

At first is require to create a message struct:  
Take a look at `message` attribute - it's a require macro to help registering and use the message inside whole message bus machinery.

```rust
#[message]
#[derive(Deserialize, Serialize)]
struct TestMessage {
    data: String,
}
```

Second step is to create message handler. This one could be in two versions both sync and async (tokio runtime)
## Sync version:
```rust
struct TestMessageHandler {}

impl MessageHandler<TestMessage> for TestMessageHandler {
    fn handle(&mut self, msg: TestMessage, headers: Option<HashMap<String, String>>) {
        println!("test {} {:?}", msg.data, headers);
    }
}
```

## Async version:
```rust
struct TestMessageHandlerAsync {}

#[async_trait]
impl MessageHandlerAsync<TestMessage> for TestMessageHandlerAsync {
    async fn handle(&mut self, msg: TestMessage, headers: Option<HashMap<String, String>>) {
        println!("test {} {:?}", msg.data, headers);
    }
}
```

# Listener
To create a Listener instance, first a pubsub client is required. For redis implemention from `bus_rs_redis` crate take a look for:
```rust
let redis_client = RedisClient::new("redis://127.0.0.1:6379", "test_channel");
let client = Box::new(redis_client);
let mut listener: Listener = builder::pubsub(client).build();
```

**Next thing, which is important, is to register message handlers to the listener:**
```rust
listener.register_handler(TestMessageHandler {});
```
thanks this a coming message could be recognize and redirect to the properly message handler.

**The last step is to make listener to listening:**
```rust
listener.listen().unwrap_or_else(|e| {
    if let ClientError::General(err) = e {
        panic!("client_error: {}", err);
    }
});
```

## Example for async listener
```rust
let redis_client = RedisClientAsync::new_receiver("redis://127.0.0.1:6379", "test_channel").await;
let client = Box::new(redis_client);
let mut listener: ListenerAsync = builder::pubsub_async(client).build();

listener.register_handler(TestMessageHandlerAsync {}).await;

// when
listener.listen().await.unwrap_or_else(|e| {
    if let ClientError::General(err) = e {
        panic!("client_error: {}", err);
    }
});
```

# Publisher
Publisher is bind to the specific pubsub channel and gives possibility to send messages.

At the beginning, a publisher instance should be created. It's a similar approach to the Listener - create Client and pass to the Publisher constructor:
```rust
let redis_client = RedisClient::new("redis://127.0.0.1:6379", "test_channel");
let client = Box::new(redis_client));
let publisher: Publisher = builder::pubsub(client).build();
```

The last step is to send message. It's worth to mention that when we wrapped a message struct before by `#[message]` attribute that give us
mapper implementation of message type to the `RawMessage`.
```rust
let test_msg = TestMessage {
    data: "test_data".to_string(),
};

publisher.publish(&test_msg, None);
```

Publish message with additional headers:
```rust
let headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
publisher.publish(&test_msg, Some(headers));
```

>> async version:
```rust
let redis_client = RedisClientAsync::new_sender("redis://127.0.0.1:6379", "test_channel").await;
let client = Box::new(redis_client));
let publisher: PublisherAsync = builder::pubsub_async(client).build();

let test_msg = TestMessage {
    data: "test_data".to_string(),
};

// when
publisher.publish(&test_msg, None).await;
```

Publish message with additional headers:
```rust
let headers = HashMap::from([("trace-id".to_owned(), "trace123".to_owned())]);
publisher.publish(&test_msg, Some(headers)).await;
```

# Layers
In a builder stage you can attach interceptors (layers). The PubSubLayer is a trait with two function to implement: before and after.
As the name suggest `before` function is calling before send/receive a message, and `after` is calling when the message sent/received.  

```rust
struct TestLayer;

impl PubSubLayer for TestLayer {
    fn before(&self, raw_msg: &mut bus_rs::RawMessage) {
        println!("Test layer before");
    }

    fn after(&self, raw_msg: &bus_rs::RawMessage) {
        println!("Test layer after");
    }
}

...

let test_layer = TestLayer {};
let mut listener: Listener = builder::pubsub(client)
    .add_layer(Box::new(test_layer))
    .build();
```
