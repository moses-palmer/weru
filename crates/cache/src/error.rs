/// The possible errors when interacting with this crate.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// Accessing a value failed.
    #[error("failed to access value: {0}")]
    ValueAccess(String),

    /// A key or a value could not be encoded or decoded.
    #[error("encoding/decoding a key or value failed: {0}")]
    Encoding(String),
}

#[cfg(feature = "local")]
pub mod local {
    use std::sync::PoisonError;

    impl<T> From<PoisonError<T>> for super::Error {
        fn from(source: PoisonError<T>) -> Self {
            Self::ValueAccess(source.to_string())
        }
    }
}

#[cfg(feature = "_cbor")]
pub mod cbo4ii {
    use cbor4ii::serde::{DecodeError, EncodeError};

    impl<E> From<DecodeError<E>> for super::Error
    where
        E: ::std::fmt::Debug,
    {
        fn from(source: DecodeError<E>) -> Self {
            Self::Encoding(source.to_string())
        }
    }

    impl<E> From<EncodeError<E>> for super::Error
    where
        E: ::std::fmt::Debug,
    {
        fn from(source: EncodeError<E>) -> Self {
            Self::Encoding(source.to_string())
        }
    }
}
