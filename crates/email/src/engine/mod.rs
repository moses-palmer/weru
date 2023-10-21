//! # The email engines
//!
//! This module contains the [`Engine`](Engine) used to construct actual email
//! senders. An engine is created from a configuration instance.

use lettre::message::Mailbox;

use crate::sender::LettreSender;
use crate::template::{Language, Templates};
use crate::{configuration, Configuration, Sender};

pub mod backends;

/// An engine that produces caches.
#[derive(Debug)]
pub struct Engine {
    /// The sender of e-mails.
    pub from: Mailbox,

    /// The default language to use.
    pub default_language: Language,

    /// The templates.
    pub templates: Templates,

    /// The transport backend to use.
    pub transport: Transport,
}

#[derive(Debug)]
pub enum Transport {
    /// No transport.
    #[cfg(feature = "drop")]
    Drop(backends::drop::Transport),
}

impl Engine {
    /// Attempts to create a cache.
    ///
    /// # Arguments
    /// *  `name` - The cache name.
    pub async fn sender(&self) -> Box<dyn Sender> {
        use Transport::*;
        match &self.transport {
            #[cfg(feature = "drop")]
            Drop(c) => Box::new(LettreSender::new(
                self.from.clone(),
                self.templates.clone(),
                self.default_language.clone(),
                backends::drop::Transport::from(c.clone()),
            )),
        }
    }
}

impl Configuration {
    /// Constructs a cache engine from this configuration.
    pub async fn engine(&self) -> Result<Engine, configuration::Error> {
        use crate::configuration::Transport::*;
        let from = self.from.clone();
        let default_language = self.templates.default_language.clone();
        let templates = Templates::load(&self.templates.path)?;
        let transport = match &self.transport {
            #[cfg(feature = "drop")]
            Drop(c) => Transport::Drop(c.transport().await?),
        };
        Ok(Engine {
            from,
            default_language,
            templates,
            transport,
        })
    }
}
