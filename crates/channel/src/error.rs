/// The possible errors when interacting with this crate.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("failed to connect: {0}")]
    Connection(String),

    /// Accessing a value failed.
    #[error("failed to access value: {0}")]
    ValueAccess(String),

    /// A key or a value could not be encoded or decoded.
    #[error("encoding/decoding a key or value failed: {0}")]
    Encoding(String),
}

#[cfg(feature = "local")]
pub mod local {
    use std::sync::mpsc::RecvError;
    use std::sync::PoisonError;

    impl<T> From<PoisonError<T>> for super::Error {
        fn from(source: PoisonError<T>) -> Self {
            Self::ValueAccess(source.to_string())
        }
    }

    impl From<RecvError> for super::Error {
        fn from(source: RecvError) -> Self {
            Self::Connection(source.to_string())
        }
    }
}
