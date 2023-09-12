//! # The configuration
//!
//! The [configuration struct](Configuration) provides a way to load a database
//! configuration from disk, and allows constructing a [database
//! engine](crate::engine::Engine).

use serde::{Deserialize, Serialize};

/// A configuration error.
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("configuration error: {0}")]
pub struct Error(String);

/// A serialised configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The connection string.
    pub connection_string: String,
}

impl From<sqlx::Error> for Error {
    fn from(source: sqlx::Error) -> Self {
        Self(source.to_string())
    }
}
