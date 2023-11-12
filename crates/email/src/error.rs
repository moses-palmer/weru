use crate::template::TemplateName;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The template does not exist.
    #[error("unknown template: {0}")]
    UnknownTemplate(TemplateName),

    /// The content to send is invalid.
    #[error("invalid content: {0}")]
    Content(lettre::error::Error),

    /// An error occurred when attempting to send the email.
    #[error("failed to send e-mail: {0}")]
    Transport(String),
}
