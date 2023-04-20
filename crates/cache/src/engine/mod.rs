//! # The cache engines
//!
//! This module contains the [`Engine`](Engine) used to construct actual cache
//! instances. An engine is created from a configuration instance.

use crate::{
    configuration, Cache, CacheProducer, Configuration, Error, Key, Name, Value,
};

pub mod backends;

/// An engine that produces caches.
#[derive(Debug)]
pub enum Engine {
    /// A [local cache](backends::local::Cache) backed by a hash
    /// map.
    #[cfg(feature = "local")]
    Local(backends::local::Engine),

    /// A [Redis cache](backends::redis::Cache).
    #[cfg(feature = "redis")]
    Redis(backends::redis::Engine),
}

impl Engine {
    /// Attempts to create a cache.
    ///
    /// # Arguments
    /// *  `name` - The cache name.
    pub async fn cache<K, V>(
        &self,
        name: impl Name,
    ) -> Result<Box<dyn Cache<K, V>>, Error>
    where
        K: Key,
        V: Value,
    {
        match self {
            #[cfg(feature = "local")]
            Engine::Local(engine) => engine.cache(name).await,

            #[cfg(feature = "redis")]
            Engine::Redis(engine) => engine.cache(name).await,
        }
    }
}

impl Configuration {
    /// Constructs a cache engine from this configuration.
    pub async fn engine(&self) -> Result<Engine, configuration::Error> {
        match self {
            #[cfg(feature = "local")]
            Configuration::Local(c) => c.engine().await,

            #[cfg(feature = "redis")]
            Configuration::Redis(c) => c.engine().await,
        }
    }
}
