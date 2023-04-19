//! # The Redis cache
//!
//! A Redis cache is a cache backend by Redis. It can be shared by multiple
//! processes, or even multiple computers.
//!
//! The minimum version of Redis reqired is 6.2.

use std::time::Duration;

use async_trait::async_trait;
use mobc::{Connection, Manager, Pool};
use redis::aio::{ConnectionLike, MultiplexedConnection};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};

use crate::{configuration, CacheProducer, Error, Key, Name, Value};

/// The configuration for a local cache.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The prefix used for keys.
    pub prefix: String,

    /// The Redis connection string.
    ///
    /// This is a string on the format `"redis://host:port"`.
    pub connection_string: String,
}

impl Configuration {
    /// Constructs a cache engine from this configuration.
    pub async fn engine(&self) -> Result<crate::Engine, configuration::Error> {
        let prefix = self.prefix.clone();
        let client = Client::open(self.connection_string.clone())?;
        let pool = Pool::builder().build(ConnectionManager { client });
        Ok(crate::Engine::Redis(Engine { prefix, pool }))
    }
}

/// An engine creating Redis cache instances.
pub struct Engine {
    /// The prefix used for keys.
    prefix: String,

    /// The connection pool.
    pool: Pool<ConnectionManager>,
}

impl ::std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Engine {{ prefix: {} }}", self.prefix)
    }
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
        let prefix = format!("{}{}", self.prefix, name.as_ref()).into_bytes();
        let pool = self.pool.clone();
        Ok(Box::new(Cache {
            prefix,
            pool,
            _m: ::std::marker::PhantomData,
        }))
    }
}

/// A local cache backed by a hash map.
///
/// Values are not automatically expired, only removed upon access.
pub struct Cache<K, V>
where
    K: Key,
    V: Value,
{
    /// A prefix prepended to key names.
    prefix: Vec<u8>,

    /// The connection pool.
    pool: Pool<ConnectionManager>,

    _m: ::std::marker::PhantomData<(K, V)>,
}

impl<K, V> Cache<K, V>
where
    K: Key,
    V: Value,
{
    /// Retrieves a connection from the connection pool.
    async fn connection(&self) -> Result<Connection<ConnectionManager>, Error> {
        Ok(self.pool.get().await?)
    }

    /// Generates the Redis key to use for a key name.
    ///
    /// # Arguments
    /// *  `key` - The name of the key.
    fn key_serialize(&self, key: &K) -> Result<Vec<u8>, Error> {
        let key = cbor4ii::serde::to_vec(Vec::new(), key)?;
        Ok({
            let mut result = self.prefix.clone();
            result.extend(&key);
            result
        })
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
        let mut conn = self.connection().await?;
        let key = self.key_serialize(key)?;

        match conn.get(&key).await? {
            redis::Value::Data(value) => {
                Ok(Some(self.value_deserialize(&value)?))
            }
            _ => Ok(None),
        }
    }

    async fn pop(&self, key: &K) -> Result<Option<V>, Error> {
        let mut conn = self.connection().await?;
        let key = self.key_serialize(key)?;

        match conn.get_del(&key).await? {
            redis::Value::Data(value) => {
                Ok(Some(self.value_deserialize(&value)?))
            }
            _ => Ok(None),
        }
    }

    async fn put(&self, key: K, value: V, ttl: Duration) -> Result<(), Error> {
        let mut conn = self.connection().await?;
        let key = self.key_serialize(&key)?;
        let value = self.value_serialize(&value)?;

        conn.req_packed_command(
            redis::Cmd::new()
                .arg("SET")
                .arg(key)
                .arg(value)
                .arg("PX")
                .arg(ttl.as_millis() as usize),
        )
        .await?;

        Ok(())
    }

    async fn replace(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<Option<V>, Error> {
        let mut conn = self.connection().await?;
        let key = self.key_serialize(&key)?;
        let value = self.value_serialize(&value)?;

        let previous = if let Some(ttl) = ttl {
            conn.req_packed_command(
                redis::Cmd::new()
                    .arg("SET")
                    .arg(key)
                    .arg(value)
                    .arg("XX")
                    .arg("GET")
                    .arg("PX")
                    .arg(ttl.as_millis() as usize),
            )
            .await?
        } else {
            conn.req_packed_command(
                redis::Cmd::new()
                    .arg("SET")
                    .arg(key)
                    .arg(value)
                    .arg("XX")
                    .arg("GET")
                    .arg("KEEPTTL"),
            )
            .await?
        };

        Ok(match previous {
            redis::Value::Data(value) => Some(self.value_deserialize(&value)?),
            _ => None,
        })
    }
}

/// A Redis connection manager.
#[derive(Debug)]
struct ConnectionManager {
    /// The client providing connections.
    client: Client,
}

#[async_trait]
impl Manager for ConnectionManager {
    type Connection = MultiplexedConnection;
    type Error = redis::RedisError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let c = self.client.get_multiplexed_async_connection().await?;
        Ok(c)
    }

    async fn check(
        &self,
        mut conn: Self::Connection,
    ) -> Result<Self::Connection, Self::Error> {
        redis::cmd("PING").query_async(&mut conn).await?;
        Ok(conn)
    }
}

#[cfg(test)]
fn configuration() -> Option<crate::Configuration> {
    option_env!("WERU_CACHE_REDIS").map(|connection_string| {
        crate::Configuration::Redis(Configuration {
            connection_string: connection_string.into(),
            prefix: "test".to_string(),
        })
    })
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
