//! # The Redis channel
//!
//! A Redis channel is a channel backed by Redis. It can be shared by multiple
//! processes, or even multiple computers.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use mobc::{Manager, Pool};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};

use crate::{configuration, ChannelProducer, Error, Event, Topic};

/// The configuration for a Redis channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The prefix used for channels.
    pub prefix: String,

    /// The Redis host.
    pub connection_string: String,
}

impl Configuration {
    /// Constructs a channel engine from this configuration.
    pub async fn engine(&self) -> Result<crate::Engine, configuration::Error> {
        let prefix = self.prefix.clone();
        let client = Client::open(self.connection_string.clone())?;
        let pool = Pool::builder().build(ConnectionManager {
            client: client.clone(),
        });
        Ok(crate::Engine::Redis(Engine {
            prefix,
            pool,
            client,
        }))
    }
}

/// An engine creating Redis channel instances.
pub struct Engine {
    /// The prefix used for channels.
    prefix: String,

    /// The Redis client.
    client: Client,

    /// The connection pool.
    pool: Pool<ConnectionManager>,
}

impl ::std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Engine {{ prefix: {} }}", self.prefix)
    }
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
        Ok(Box::new(Channel {
            client: self.client.clone(),
            channel: format!("{}{}", self.prefix, topic),
            pool: self.pool.clone(),
            _m: ::std::marker::PhantomData,
        }))
    }
}

/// A Redis channel.
#[derive(Clone)]
pub struct Channel<T>
where
    T: Event,
{
    // The Redis client.
    client: Client,

    /// The name of the channel.
    channel: String,

    /// The connection pool.
    pool: Pool<ConnectionManager>,

    _m: ::std::marker::PhantomData<T>,
}

#[async_trait]
impl<T> crate::Channel<T> for Channel<T>
where
    T: Event,
{
    async fn broadcast(&self, event: T) -> Result<(), Error> {
        let mut conn = self.pool.get().await?;
        let bytes = cbor4ii::serde::to_vec(Vec::new(), &event)?;
        Ok(conn.publish(&self.channel, bytes).await?)
    }

    async fn listen(
        &self,
    ) -> Result<BoxStream<'static, Result<T, Error>>, Error> {
        let mut pubsub =
            self.client.get_async_connection().await?.into_pubsub();
        pubsub.subscribe(&self.channel).await?;
        Ok(Box::pin(pubsub.into_on_message().map(|msg| {
            Ok(cbor4ii::serde::from_slice(msg.get_payload_bytes())?)
        })))
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
    option_env!("WERU_CHANNEL_REDIS").map(|connection_string| {
        crate::Configuration::Redis(Configuration {
            connection_string: connection_string.into(),
            prefix: "test".to_string(),
        })
    })
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
