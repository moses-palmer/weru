use serde::{Deserialize, Serialize};

pub use actix_session::storage::RedisSessionStore as Store;

use crate::configuration;

/// The size, in bytes, of a key used to protect sessions.
const SESSION_KEY_SIZE: usize = 32 + 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The secret used to protect cookies.
    pub secret: super::Secret<SESSION_KEY_SIZE>,

    /// The name of the cookie
    pub name: String,

    /// The Redis connection string.
    ///
    /// This is a string on the format `"redis://host:port"`.
    pub connection_string: String,

    /// The time-to-live for cookies, in seconds.
    pub ttl: u32,

    /// The prefix used to generate the Redis keys for sessions.
    pub key_prefix: String,
}

impl Configuration {
    /// Constructs a session middleware with this configuraed storage.
    pub async fn store(&self) -> Result<Store, configuration::Error> {
        let key_prefix = self.key_prefix.clone();
        Ok(Store::builder(&self.connection_string)
            .cache_keygen(move |key| format!("{}.{}", key_prefix, key))
            .build()
            .await?)
    }
}

/// Creates a clone of a store.
///
/// # Arguments
/// *  `store` - The store to clone.
pub fn clone(store: &Store) -> Store {
    store.clone()
}
