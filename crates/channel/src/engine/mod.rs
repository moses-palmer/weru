//! # The channel engines
//!
//! This module contains the [`Engine`](Engine) used to construct actual
//! channel instances. An engine is created from a configuration instance.

use crate::{
    configuration, Channel, ChannelProducer, Configuration, Error, Event, Topic,
};

pub mod backends;

/// An engine that produces channels.
#[derive(Debug)]
pub enum Engine {
    /// A [local channel](backends::local::Channel) backed by in-process
    /// channels.
    #[cfg(feature = "local")]
    Local(backends::local::Engine),

    /// A [Redis channel](backends::redis::Channel).
    #[cfg(feature = "redis")]
    Redis(backends::redis::Engine),
}

impl Engine {
    /// Attempts to create a channel.
    ///
    /// # Arguments
    /// *  `topic` - The channel topic.
    pub async fn channel<T>(
        &self,
        topic: impl Topic,
    ) -> Result<Box<dyn Channel<T>>, Error>
    where
        T: Event,
    {
        match self {
            #[cfg(feature = "local")]
            Engine::Local(engine) => engine.channel(topic).await,

            #[cfg(feature = "redis")]
            Engine::Redis(engine) => engine.channel(topic).await,
        }
    }
}

impl Configuration {
    /// Constructs a channel engine from this configuration.
    pub async fn engine(&self) -> Result<Engine, configuration::Error> {
        match self {
            #[cfg(feature = "local")]
            Configuration::Local(c) => c.engine().await,

            #[cfg(feature = "redis")]
            Configuration::Redis(c) => c.engine().await,
        }
    }
}
