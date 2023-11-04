//! # An SMTP e-mail sender
//!
//! This sender uses the SMTP protocol.

use lettre::transport::smtp::{AsyncSmtpTransport, AsyncSmtpTransportBuilder};
use serde::{Deserialize, Serialize};

pub use lettre::transport::smtp::authentication::Mechanism;

use crate::configuration::Error;

pub type Transport = AsyncSmtpTransport<lettre::Tokio1Executor>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ConnectMethod {
    /// Connect using simple TLS.
    TLS,

    /// Connect by first using an unencrypted channel, and then upgrading using
    /// STARTTLS.
    StartTLS,
}

/// The configuration for the dummy sender.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The SMTP host.
    server: String,

    /// The port.
    port: u16,

    /// The method to use to establish connections.
    connect_method: ConnectMethod,

    /// The connection user name.
    username: String,

    /// The connection password.
    password: String,

    /// The authentication mechanisms to use.
    mechanisms: Vec<Mechanism>,
}

impl Configuration {
    /// Constructs a transport from this configuration.
    pub async fn transport(&self) -> Result<Transport, Error> {
        Ok(self
            .connect_method
            .with_host(&self.server)?
            .port(self.port)
            .credentials((&self.username, &self.password).into())
            .authentication(self.mechanisms.clone())
            .build())
    }
}

impl ConnectMethod {
    /// Constructs a builder for this connect method.
    ///
    /// # Arguments
    /// *  `host` - The hostname for the connection.
    fn with_host(self, host: &str) -> Result<AsyncSmtpTransportBuilder, Error> {
        use ConnectMethod::*;
        match self {
            TLS => Ok(Transport::relay(host)?),
            StartTLS => Ok(Transport::starttls_relay(host)?),
        }
    }
}
