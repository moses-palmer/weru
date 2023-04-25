use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::Error;

/// A channel topic.
pub trait Topic: Send + Sync + ::std::fmt::Display {}

impl<T> Topic for T where T: Send + Sync + ::std::fmt::Display {}

/// An event.
pub trait Event:
    Clone + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>
{
}

impl<T> Event for T where
    T: Clone + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>
{
}

/// A channel on which to listen and broadcast events.
///
/// # Argument
/// *  `T` - The type for events.
#[async_trait]
pub trait Channel<T>: Send + Sync
where
    T: Event,
{
    /// Broadcasts an event on this channel.
    ///
    /// # Arguments
    /// *  `event` - The event to broadcast.
    async fn broadcast(&self, event: T) -> Result<(), Error>;

    /// Listens on this channel.
    async fn listen(
        &self,
    ) -> Result<BoxStream<'static, Result<T, Error>>, Error>;
}

/// A channel producing engine.
#[async_trait]
pub trait ChannelProducer {
    /// Attempts to create a channel.
    ///
    /// # Arguments
    /// *  `topic` - The topic name.
    async fn channel<T>(
        &self,
        topic: impl Topic,
    ) -> Result<Box<dyn Channel<T>>, Error>
    where
        T: Event;
}
