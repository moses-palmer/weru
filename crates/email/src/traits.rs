use std::collections::HashMap;

use async_trait::async_trait;
use lettre::message::Mailboxes;

use crate::template::{Language, TemplateName};
use crate::Error;

/// An e-mail sender.
#[async_trait]
pub trait Sender: Send + Sync {
    /// Sends an e-mail to a recipient.
    ///
    /// # Arguments
    /// *  `recipients`- The e-mail recipients.
    /// *  `languages` - A sequence of langauges to use, in decreasing order of
    ///    relevance. The first langauge for which the template exists is used.
    /// *  `template` - The template to use for format the message.
    /// *  `replacements` - A function converting keys to replacement strings.
    ///    If this function returns `None`, the replacement string is kept.
    async fn send(
        &self,
        recipients: Mailboxes,
        languages: &[Language],
        template: &TemplateName,
        replacements: &HashMap<String, String>,
    ) -> Result<(), Error>;
}
