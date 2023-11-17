use serde::{Deserialize, Serialize};

pub use actix_session::storage::CookieSessionStore as Store;

use crate::configuration;

/// The size, in bytes, of a key used to protect sessions.
const SESSION_KEY_SIZE: usize = 32 + 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The secret used to protect cookies.
    pub secret: super::Secret<SESSION_KEY_SIZE>,

    /// The name of the cookie
    pub name: String,
}

impl Configuration {
    /// Constructs a session middleware with this configuraed storage.
    pub async fn store(&self) -> Result<Store, configuration::Error> {
        Ok(Store::default())
    }
}

/// Creates a clone of a store.
///
/// # Arguments
/// *  `_store` - The store to clone.
pub fn clone(_store: &Store) -> Store {
    Store::default()
}
