//! # The local cache
//!
//! A local cache is a cache backend by a simple
//! [HashMap](::std::collections::HashMap). It can only be used in the local
//! process.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{configuration, CacheProducer, Error, Key, Name, Value};

/// The configuration for a local cache.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration;

impl Configuration {
    /// Constructs a cache engine from this configuration.
    pub async fn engine(&self) -> Result<crate::Engine, configuration::Error> {
        Ok(crate::Engine::Local(Engine {
            data: Arc::new(Mutex::new(HashMap::new())),
        }))
    }
}

/// An engine creating local cache instances.
#[derive(Debug)]
pub struct Engine {
    /// The buffers of serialised data.
    data: Arc<Mutex<HashMap<String, Arc<Mutex<Buffer>>>>>,
}

#[async_trait]
impl CacheProducer for Engine {
    async fn cache<K, V>(
        &self,
        name: impl Name,
    ) -> Result<Box<dyn crate::Cache<K, V>>, Error>
    where
        K: Key,
        V: Value,
    {
        let mut data = self.data.lock()?;

        Ok(Box::new(Cache::new(Arc::clone(
            data.entry(name.as_ref().to_string())
                .or_insert_with(|| Arc::new(Mutex::new(Buffer::new()))),
        ))))
    }
}

/// A local cache backed by a hash map.
///
/// Values are not automatically expired, only removed upon access.
pub struct Cache<K, V> {
    /// The data.
    data: Arc<Mutex<Buffer>>,

    _m: ::std::marker::PhantomData<(K, V)>,
}

impl<K, V> Cache<K, V> {
    /// Creates a new cache with a backing buffer.
    ///
    /// # Arguments
    /// *  `data` The backing buffer.
    fn new(data: Arc<Mutex<Buffer>>) -> Self {
        Self {
            data,
            _m: ::std::marker::PhantomData,
        }
    }
}

impl<K, V> Cache<K, V>
where
    K: Key,
    V: Value,
{
    /// Generates the Redis key to use for a key name.
    ///
    /// # Arguments
    /// *  `key` - The name of the key.
    fn key_serialize(&self, key: &K) -> Result<Vec<u8>, Error> {
        Ok(cbor4ii::serde::to_vec(Vec::new(), key)?)
    }

    /// Converts a value to a byte vector.
    ///
    /// # Arguments
    /// *  `value` - The value to convert.
    fn value_serialize(&self, value: &V) -> Result<Vec<u8>, Error> {
        Ok(cbor4ii::serde::to_vec(Vec::new(), value)?)
    }

    /// Converts a byte vector to a value.
    ///
    /// # Arguments
    /// *  `value` - The value to convert.
    fn value_deserialize<T>(&self, value: &[u8]) -> Result<T, Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        Ok(cbor4ii::serde::from_slice(value)?)
    }
}

#[async_trait]
impl<K, V> crate::Cache<K, V> for Cache<K, V>
where
    K: Key,
    V: Value,
{
    async fn get(&self, key: &K) -> Result<Option<V>, Error> {
        let mut data = self.data.lock()?;
        let key = self.key_serialize(key)?;

        data.get(&key)
            .map(|bytes| Ok(self.value_deserialize(&bytes)?))
            .transpose()
    }

    async fn pop(&self, key: &K) -> Result<Option<V>, Error> {
        let mut data = self.data.lock()?;
        let key = self.key_serialize(key)?;

        data.remove(&key)
            .map(|d| Ok(self.value_deserialize(&d.value)?))
            .transpose()
    }

    async fn put(&self, key: K, value: V, ttl: Duration) -> Result<(), Error> {
        let mut data = self.data.lock()?;
        let key = self.key_serialize(&key)?;
        let value = self.value_serialize(&value)?;

        data.put(key, value, Instant::now() + ttl);
        Ok(())
    }

    async fn replace(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<Option<V>, Error> {
        let mut data = self.data.lock()?;
        let key = self.key_serialize(&key)?;
        let value = self.value_serialize(&value)?;

        if let Some(d) = data.remove(&key) {
            data.put(
                key,
                value,
                ttl.map(|ttl| Instant::now() + ttl).unwrap_or(d.expiry),
            );
            Some(self.value_deserialize(&d.value)).transpose()
        } else {
            Ok(None)
        }
    }
}

/// An untyped cache.
#[derive(Debug)]
struct Buffer {
    /// The data.
    data: HashMap<Vec<u8>, Data>,
}

impl Buffer {
    /// Creates a new empty buffer.
    pub fn new() -> Self {
        Buffer {
            data: HashMap::new(),
        }
    }

    /// Reads a value from the buffer.
    ///
    /// If the value has expired, it is removed and nothing is returned.
    ///
    /// # Arguments
    /// *  `key` - The key to read.
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let now = Instant::now();

        match self.data.get(key) {
            Some(w) => {
                if w.live(now) {
                    Some(w.clone_inner())
                } else {
                    self.data.remove(key);
                    None
                }
            }
            None => None,
        }
    }

    /// Removes a value from the buffer and returnes it.
    ///
    /// If the value has expired, it is removed and nothing is returned.
    ///
    /// # Arguments
    /// *  `key` - The key to remove.
    pub fn remove(&mut self, key: &[u8]) -> Option<Data> {
        let now = Instant::now();

        match self.data.remove(key) {
            Some(w) => {
                if w.live(now) {
                    Some(w)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Writes a value to the buffer.
    ///
    /// # Arguments
    /// *  `key` - The key to write.
    /// *  `value` - The value to write.
    /// *  `ttl` - The time-to-live.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>, expiry: Instant) {
        self.data.insert(key, Data::new(value, expiry));
    }
}
/// A wrapper for a value with a time-to-live.
#[derive(Clone, Debug)]
struct Data {
    /// The value.
    value: Vec<u8>,

    /// The instant of expiry.
    expiry: Instant,
}

impl Data {
    /// Creates a new wrapper.
    ///
    /// # Arguments
    /// *  `value` - The value to wrap.
    /// *  `expiry` - The instant of expiry for the value.
    pub fn new(value: Vec<u8>, expiry: Instant) -> Self {
        Self { value, expiry }
    }

    /// Checks the expiry of this wrapper, and returns whether it is still
    /// live.
    ///
    /// # Arguments
    /// *  `expiry` - The expiry time to check.
    pub fn live(&self, expiry: Instant) -> bool {
        expiry < self.expiry
    }

    /// Clones the inner value.
    pub fn clone_inner(&self) -> Vec<u8> {
        self.value.clone()
    }
}

#[cfg(test)]
fn configuration() -> Option<crate::Configuration> {
    Some(crate::Configuration::Local(Configuration))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
