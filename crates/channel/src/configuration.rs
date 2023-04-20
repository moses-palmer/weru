//! # The configuration
//!
//! The [configuration struct](Configuration) provides a way load a
//! configuration from disk, and allows constructing a [channel
//! engine](crate::engine::Engine).

use serde::{Deserialize, Serialize};

/// A configuration error.
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("configuration error: {0}")]
pub struct Error(String);

/// A serialised configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Configuration {
    /// A local cache backed by an in-process channel.
    #[cfg(feature = "local")]
    Local(crate::engine::backends::local::Configuration),
}
