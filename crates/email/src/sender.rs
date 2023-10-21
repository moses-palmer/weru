use std::collections::HashMap;
use std::iter;

use async_trait::async_trait;
use lettre;

use lettre::message::{header, Attachment, Message, MultiPart, SinglePart};
use lettre::AsyncTransport;

use crate::error::Error;
use crate::template::{Language, TemplateName, Templates};
use crate::Sender;

pub use lettre::message::{Mailbox, Mailboxes};

/// A sender of emails.
///
/// This struct maintains a selection of templates
pub struct LettreSender<T>
where
    T: AsyncTransport + Sync,
{
    /// The templates used by this email sender.
    templates: Templates,

    /// The mailbox indicated by the _From_ header.
    from: Mailbox,

    /// The default language to use when none of the requested languages is
    /// available.
    default_language: Language,

    /// The actual email transport method.
    transport: T,
}

impl<T> LettreSender<T>
where
    T: AsyncTransport + Sync,
{
    /// Creates a new email sender.
    /// *  `from` - The mailbox indicated by the _From_ header.
    /// *  `templates` - The templates used by this email sender.
    /// *  `default_language` - The default language to use when none of the
    ///    requested languages is available.
    /// *  `transport` - The actual email transport method.
    pub fn new(
        from: Mailbox,
        templates: Templates,
        default_language: Language,
        transport: T,
    ) -> Self {
        Self {
            templates,
            from,
            default_language,
            transport,
        }
    }
}

#[async_trait]
impl<T> Sender for LettreSender<T>
where
    T: AsyncTransport + Send + Sync,
    <T as AsyncTransport>::Error: std::fmt::Display,
{
    async fn send(
        &self,
        recipients: Mailboxes,
        languages: &[Language],
        template: &TemplateName,
        replacements: &HashMap<String, String>,
    ) -> Result<(), Error> {
        let template = languages
            .iter()
            .chain(iter::once(&self.default_language))
            .find_map(|language| self.templates.get(&language, template))
            .ok_or_else(|| Error::UnknownTemplate(template.clone()))?;
        let message = Message::builder()
            .from(self.from.clone())
            .subject(template.subject())
            .mailbox(header::To::from(recipients))
            .multipart(
                template.attachments().iter().fold(
                    MultiPart::related()
                        .singlepart(SinglePart::html(template.html(|key| {
                            replacements.get(key).map(String::as_str)
                        })))
                        .singlepart(SinglePart::plain(template.text(|key| {
                            replacements.get(key).map(String::as_str)
                        }))),
                    |multipart, (name, attachment)| {
                        multipart.singlepart(
                            Attachment::new_inline(name.as_ref().clone()).body(
                                attachment.data().to_vec(),
                                attachment.content_type().clone(),
                            ),
                        )
                    },
                ),
            )
            .map_err(Error::Content)?;

        Ok(self
            .transport
            .send(message)
            .await
            .map(|_| ())
            .map_err(|e| Error::Transport(e.to_string()))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use lettre::transport::stub::AsyncStubTransport;

    use crate::template::Templates;

    #[actix_rt::test]
    async fn fails_for_unknown() {
        let sender = LettreSender::new(
            "Sender <sender@domain>".parse().unwrap(),
            templates(),
            "default".into(),
            AsyncStubTransport::new_ok(),
        );

        assert_eq!(
            Err(Error::UnknownTemplate("tx".into()).to_string()),
            sender
                .send(
                    recipients(),
                    &["l1".into()],
                    &"tx".into(),
                    &replacements()
                )
                .await
                .map_err(|e| e.to_string()),
        );
    }

    #[actix_rt::test]
    async fn fails_for_transport_error() {
        let sender = LettreSender::new(
            "Sender <sender@domain>".parse().unwrap(),
            templates(),
            "default".into(),
            AsyncStubTransport::new_error(),
        );

        assert_eq!(
            Err(Error::Transport("stub error".into()))
                .map_err(|e| e.to_string()),
            sender
                .send(
                    recipients(),
                    &["l1".into()],
                    &"t1".into(),
                    &replacements()
                )
                .await
                .map_err(|e| e.to_string())
        );
    }

    #[actix_rt::test]
    async fn succeeds_for_no_error() {
        let sender = LettreSender::new(
            "Sender <sender@domain>".parse().unwrap(),
            templates(),
            "default".into(),
            AsyncStubTransport::new_ok(),
        );

        assert_eq!(
            Ok(()),
            sender
                .send(
                    recipients(),
                    &["l1".into()],
                    &"t1".into(),
                    &replacements()
                )
                .await
                .map_err(|e| e.to_string()),
        );
    }

    /// Loads the valid templates from the test resource directory.
    fn templates() -> Templates {
        Templates::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/test/email/template/valid.toml"),
        )
        .unwrap()
    }

    /// A simple mailbox.
    fn recipients() -> Mailboxes {
        "Tester <test@test.com>".parse().unwrap()
    }

    /// A no-op replacements map.
    fn replacements() -> HashMap<String, String> {
        HashMap::new()
    }
}
