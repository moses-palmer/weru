//! # The silent e-mail sender
//!
//! A dummy e-mail sender that always drops e-mails.

use serde::{Deserialize, Serialize};

pub use lettre::transport::stub::AsyncStubTransport as Transport;

use crate::configuration::Error;

/// The configuration for the dummy sender.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration;

impl Configuration {
    /// Constructs a transport from this configuration.
    pub async fn transport(&self) -> Result<Transport, Error> {
        Ok(Transport::new_ok())
    }
}
