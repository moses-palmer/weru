use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

use crate::template::{Error as TemplateError, Language};

/// A configuration error.
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("configuration error: {0}")]
pub struct Error(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The sender of e-mails.
    pub from: Mailbox,

    /// The templates.
    pub templates: Templates,

    /// The transport backend to use.
    pub transport: Transport,
}

/// Contains information about templates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Templates {
    /// The default language to use when none of the requested languages are
    /// available.
    ///
    /// A template for this language should always be available.
    pub default_language: Language,

    /// The path to the template description file.
    pub path: String,
}
///
/// Contains information about templates.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Transport {
    /// No transport.
    ///
    /// Using this will drop all e-mails.
    #[cfg(feature = "drop")]
    Drop(crate::engine::backends::drop::Configuration),
}

impl From<TemplateError> for Error {
    fn from(source: TemplateError) -> Self {
        Self(source.to_string())
    }
}
