//! # The local channel
//!
//! A local channel is a channel backend by a local bus. It can only be used in
//! the local process.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Poll;

use async_trait::async_trait;
use bus::Bus;
use futures::stream::{poll_fn, BoxStream};
use serde::{Deserialize, Serialize};
use type_map::concurrent::TypeMap;

use crate::{configuration, ChannelProducer, Error, Event, Topic};

/// The configuration for a local channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The size of the queue holding sent messages.
    pub queue_size: usize,
}

impl Configuration {
    /// Constructs a channel engine from this configuration.
    pub async fn engine(&self) -> Result<crate::Engine, configuration::Error> {
        Ok(crate::Engine::Local(Engine {
            queue_size: self.queue_size,
            channel_types: Arc::new(Mutex::new(TypeMap::new())),
        }))
    }
}

/// An engine creating local channel instances.
#[derive(Debug)]
pub struct Engine {
    /// The size of the queue holding sent messages.
    queue_size: usize,

    /// A map from channel types to maps of channels.
    channel_types: Arc<Mutex<TypeMap>>,
}

#[async_trait]
impl ChannelProducer for Engine {
    async fn channel<T>(
        &self,
        topic: impl Topic,
    ) -> Result<Box<dyn crate::Channel<T>>, Error>
    where
        T: Event,
    {
        let mut channel_types = self.channel_types.lock()?;

        let channels = channel_types
            .entry::<HashMap<String, Arc<Channel<T>>>>()
            .or_insert_with(HashMap::new);
        let channel = channels.entry(topic.to_string()).or_insert_with(|| {
            Arc::new(Channel {
                bus: Arc::new(Mutex::new(Bus::new(self.queue_size))),
            })
        });
        Ok(Box::new(channel.clone()))
    }
}

/// A local channel backed by a local bus.
pub struct Channel<T>
where
    T: Event,
{
    /// The bus for this channel.
    bus: Arc<Mutex<Bus<T>>>,
}

#[async_trait]
impl<T> crate::Channel<T> for Arc<Channel<T>>
where
    T: Event,
{
    async fn broadcast(&self, event: T) -> Result<(), Error> {
        let mut bus = self.bus.lock()?;

        bus.try_broadcast(event)
            .map_err(|_| Error::Connection("queue is full".into()))
    }

    async fn listen(
        &self,
    ) -> Result<BoxStream<'static, Result<T, Error>>, Error> {
        let mut bus = self.bus.lock()?;

        let mut receiver = bus.add_rx();
        Ok(Box::pin(poll_fn(move |_| match receiver.try_recv() {
            Ok(event) => Poll::Ready(Some(Ok(event))),
            Err(_) => Poll::Ready(None),
        })))
    }
}

#[cfg(test)]
fn configuration() -> Option<crate::Configuration> {
    Some(crate::Configuration::Local(Configuration {
        queue_size: 10,
    }))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
