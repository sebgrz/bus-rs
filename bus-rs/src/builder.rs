use std::sync::{Arc, Mutex};

use crate::{
    listener::Listener, listener_async::ListenerAsync, publisher::Publisher,
    publisher_async::PublisherAsync, Client, ClientAsync, PubSubLayer, PublisherContext,
    PublisherContextAsync,
};

pub trait Builder<TPubSub> {
    fn build(self) -> TPubSub;
}

pub struct PubSubBuilder {
    client: Option<Box<dyn Client + Send + Sync>>,
    client_async: Option<Box<dyn ClientAsync + Send + Sync>>,
    layers: Vec<Box<dyn PubSubLayer>>,
}

pub fn pubsub(client: Box<dyn Client + Send + Sync>) -> PubSubBuilder {
    PubSubBuilder {
        client: Some(client),
        client_async: None,
        layers: vec![],
    }
}

pub fn pubsub_async(client: Box<dyn ClientAsync + Send + Sync>) -> PubSubBuilder {
    PubSubBuilder {
        client: None,
        client_async: Some(client),
        layers: vec![],
    }
}

impl PubSubBuilder {
    pub fn add_layer(mut self, layer: Box<dyn PubSubLayer>) -> Self {
        self.layers.push(layer);
        self
    }
}

impl Builder<Listener> for PubSubBuilder {
    fn build(self) -> Listener {
        Listener::new(self.client.unwrap(), self.layers)
    }
}

impl Builder<Publisher> for PubSubBuilder {
    fn build(self) -> Publisher {
        let context = PublisherContext {
            client: self.client.unwrap(),
            layers: self.layers,
        };
        Publisher::new(Arc::new(Mutex::new(context)))
    }
}

impl Builder<ListenerAsync> for PubSubBuilder {
    fn build(self) -> ListenerAsync {
        ListenerAsync::new(self.client_async.unwrap(), self.layers)
    }
}

impl Builder<PublisherAsync> for PubSubBuilder {
    fn build(self) -> PublisherAsync {
        let context = PublisherContextAsync {
            client: self.client_async.unwrap(),
            layers: self.layers,
        };
        PublisherAsync::new(Arc::new(tokio::sync::Mutex::new(context)))
    }
}
