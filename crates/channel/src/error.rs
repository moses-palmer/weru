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

#[cfg(feature = "redis")]
pub mod redis {
    use cbor4ii::serde::{DecodeError, EncodeError};
    use mobc::Error as PoolError;
    use redis::RedisError;

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

    impl<E> From<PoolError<E>> for super::Error
    where
        E: ::std::fmt::Display,
    {
        fn from(source: PoolError<E>) -> Self {
            Self::Connection(source.to_string())
        }
    }

    impl From<RedisError> for super::Error {
        fn from(source: RedisError) -> Self {
            Self::ValueAccess(source.to_string())
        }
    }
}
