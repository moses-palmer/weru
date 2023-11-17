//! # The configuration
//!
//! The [configuration struct](Configuration) provides a way load a
//! configuration from disk, and allows constructing a [session
//! middleware](crate::SessionMiddleware).

use serde::{Deserialize, Serialize};

/// A configuration error.
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("configuration error: {0}")]
pub struct Error(String);

/// A serialised configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Configuration {
    /// A session backed by an encrypted cookie.
    #[cfg(feature = "cookie")]
    Cookie(crate::store::cookie::Configuration),
}

impl From<anyhow::Error> for Error {
    fn from(source: anyhow::Error) -> Self {
        Self(source.to_string())
    }
}
