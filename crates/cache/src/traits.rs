use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Error;

/// A cache name.
pub trait Name: Send + Sync + AsRef<str> {}

impl<T> Name for T where T: Send + Sync + AsRef<str> {}

/// A key in the cache.
pub trait Key:
    Clone
    + Eq
    + PartialEq
    + Send
    + Sync
    + ::std::hash::Hash
    + Serialize
    + 'static
    + for<'a> Deserialize<'a>
{
}

impl<T> Key for T where
    T: Clone
        + Eq
        + PartialEq
        + Send
        + Sync
        + ::std::hash::Hash
        + Serialize
        + 'static
        + for<'a> Deserialize<'a>
{
}

/// A value in the cache.
pub trait Value:
    Clone + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>
{
}

impl<T> Value for T where
    T: Clone + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>
{
}

/// A key-value cache.
///
/// # Argument
/// *  `K` - The type for keys.
/// *  `V` - The type for values.
#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: Key,
    V: Value,
{
    /// Reads a value from the cache.
    ///
    /// # Arguments
    /// *  `key` - The key to read.
    async fn get(&self, key: &K) -> Result<Option<V>, Error>;

    /// Pops a value from the cache.
    ///
    /// # Arguments
    /// *  `key` - The key to pop.
    async fn pop(&self, key: &K) -> Result<Option<V>, Error>;

    /// Writes a value to the cache.
    ///
    /// # Arguments
    /// *  `key` - The key to write.
    /// *  `value` - The value to write.
    /// *  `ttl` - The time-to-live for the value.
    async fn put(&self, key: K, value: V, ttl: Duration) -> Result<(), Error>;

    /// Replaces a value in the cache.
    ///
    /// If not value exists under the specific key, none will be written.
    ///
    /// # Arguments
    /// *  `key` - The key to write.
    /// *  `value` - The value to write.
    /// *  `ttl` - The time-to-live for the value. If this is not specified,
    ///    the previous value is reused.
    async fn replace(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<Option<V>, Error>;
}

/// A cache producing engine.
#[async_trait]
pub trait CacheProducer {
    /// Attempts to create a cache.
    ///
    /// # Arguments
    /// *  `name` - The cache name.
    async fn cache<K, V>(
        &self,
        name: impl Name,
    ) -> Result<Box<dyn Cache<K, V>>, Error>
    where
        K: Key,
        V: Value;
}
