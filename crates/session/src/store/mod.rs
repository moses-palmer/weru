//! # The storage backends
//!
//! This module contains the various kinds of storages for sessions.
use std::collections::HashMap;

use actix_session::{
    config::PersistentSession,
    storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError},
};
use actix_web::cookie::{time::Duration, Key};
use serde::{Deserialize, Serialize};

use crate::{configuration, Configuration};

pub mod cookie;
pub mod redis;

///
type SessionState = HashMap<String, String>;

/// An engine that produces channels.
pub enum Store {
    /// A storage backed by a cookie value.
    #[cfg(feature = "cookie")]
    Cookie(cookie::Store),

    /// A storage backed by Redis.
    #[cfg(feature = "redis")]
    Redis(redis::Store),
}

impl Store {
    /// Constructs a session middleware from this store and a configuration.
    ///
    /// # Arguments
    /// *  `configuration` - The configuration.
    pub fn middleware(
        self,
        configuration: &Configuration,
    ) -> crate::SessionMiddleware {
        crate::SessionMiddleware::builder(self, configuration.key())
            .cookie_name(configuration.name())
            .cookie_secure(true)
            .session_lifecycle(PersistentSession::default())
            .build()
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        use Store::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(store) => Cookie(cookie::clone(store)),
            #[cfg(feature = "redis")]
            Redis(store) => Redis(redis::clone(store)),
        }
    }
}

impl Configuration {
    /// Constructs a backing store.
    pub async fn store(&self) -> Result<Store, configuration::Error> {
        use Configuration::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(c) => c.store().await.map(Store::Cookie),
            #[cfg(feature = "redis")]
            Redis(c) => c.store().await.map(Store::Redis),
        }
    }

    /// The secret key to use.
    fn key(&self) -> Key {
        use Configuration::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(c) => Key::from(&c.secret.key),
            #[cfg(feature = "redis")]
            Redis(c) => Key::from(&c.secret.key),
        }
    }

    /// The name of the cookie.
    fn name(&self) -> String {
        use Configuration::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(c) => c.name.clone(),
            #[cfg(feature = "redis")]
            Redis(c) => c.name.clone(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for Store {
    async fn load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<SessionState>, LoadError> {
        use Store::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(s) => s.load(session_key).await,
            #[cfg(feature = "redis")]
            Redis(s) => s.load(session_key).await,
        }
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        use Store::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(s) => s.save(session_state, ttl).await,
            #[cfg(feature = "redis")]
            Redis(s) => s.save(session_state, ttl).await,
        }
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        use Store::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(s) => s.update(session_key, session_state, ttl).await,
            #[cfg(feature = "redis")]
            Redis(s) => s.update(session_key, session_state, ttl).await,
        }
    }

    async fn update_ttl(
        &self,
        session_key: &SessionKey,
        ttl: &Duration,
    ) -> Result<(), anyhow::Error> {
        use Store::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(s) => s.update_ttl(session_key, ttl).await,
            #[cfg(feature = "redis")]
            Redis(s) => s.update_ttl(session_key, ttl).await,
        }
    }

    async fn delete(
        &self,
        session_key: &SessionKey,
    ) -> Result<(), anyhow::Error> {
        use Store::*;
        match self {
            #[cfg(feature = "cookie")]
            Cookie(s) => s.delete(session_key).await,
            #[cfg(feature = "redis")]
            Redis(s) => s.delete(session_key).await,
        }
    }
}

/// A key used internally to maintain secrets.
///
/// When represented by a string, this is a string of length `SIZE * 2` of
/// hexadecimal characters. A byte is represented with the least significant
/// bits written first, so `0x12u8` will be read and written as `"21"`.
#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct Secret<const SIZE: usize> {
    /// The key.
    pub key: [u8; SIZE],
}

impl<const SIZE: usize> Secret<SIZE> {
    /// The digits used when serialising and deserialising.
    const DIGITS: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D',
        'E', 'F',
    ];
}

impl<const SIZE: usize> ::std::fmt::Debug for Secret<SIZE> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{:*<1$}", "", SIZE)
    }
}

impl<const SIZE: usize> ::std::str::FromStr for Secret<SIZE> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            key: s
                .chars()
                .map(|c| c.to_digit(16))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| format!("secret contains an invalid character"))
                .and_then(|bytes| {
                    if bytes.len() == 2 * SIZE {
                        Ok(bytes)
                    } else {
                        Err(format!(
                            "expected length of secret to be {}, found {}",
                            2 * SIZE,
                            s.len(),
                        ))
                    }
                })?
                .chunks(2)
                .map(|c| (c[1] | (c[0] << 4)) as u8)
                .collect::<Vec<_>>()
                .try_into()
                // We have explicitly checked the length before
                .unwrap(),
        })
    }
}

impl<const SIZE: usize> TryFrom<String> for Secret<SIZE> {
    type Error = String;

    /// Attempts to construct a secret from a string.
    ///
    /// if `source` is an invalid string, an error is returned.
    ///
    /// # Arguments
    /// *  `source` - The source string.
    fn try_from(source: String) -> Result<Self, Self::Error> {
        source.parse()
    }
}

impl<const SIZE: usize> Into<String> for Secret<SIZE> {
    /// Converts this secret into a hexadecimal encoded string.
    fn into(self) -> String {
        (0..SIZE)
            .map(|i| {
                let byte = self.key[i >> 1];
                if i & 1 == 0 {
                    Self::DIGITS[(byte & 0x0F) as usize]
                } else {
                    Self::DIGITS[((byte >> 4) & 0x0F) as usize]
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_parse_invalid() {
        // Arrange
        let source = "invalid";

        // Act
        let result = source.parse::<Secret<4>>();

        // Assert
        assert_eq!(Err("secret contains an invalid character".into()), result);
    }

    #[test]
    fn secret_parse_too_short() {
        // Arrange
        let source = "5f4c11";

        // Act
        let result = source.parse::<Secret<4>>();

        // Assert
        assert_eq!(
            Err("expected length of secret to be 8, found 6".into()),
            result,
        );
    }

    #[test]
    fn secret_parse_too_long() {
        // Arrange
        let source = "5f4c115f4c11";

        // Act
        let result = source.parse::<Secret<4>>();

        // Assert
        assert_eq!(
            Err("expected length of secret to be 8, found 12".into()),
            result,
        );
    }

    #[test]
    fn secret_parse_one_off() {
        // Arrange
        let source1 = "5f4c113";
        let source2 = "5f4c11345";

        // Act
        let result1 = source1.parse::<Secret<4>>();
        let result2 = source2.parse::<Secret<4>>();

        // Assert
        assert_eq!(
            Err("expected length of secret to be 8, found 7".into()),
            result1,
        );
        assert_eq!(
            Err("expected length of secret to be 8, found 9".into()),
            result2,
        );
    }

    #[test]
    fn secret_parse_ok() {
        // Arrange
        let source = "5f4c115f";

        // Act
        let result = source.parse::<Secret<4>>();

        // Assert
        assert_eq!(
            Ok(Secret {
                key: [0x5f, 0x4c, 0x11, 0x5f],
            }),
            result,
        );
    }
}
