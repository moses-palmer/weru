//! # The database engines
//!
//! This module contains the [`Engine`](Engine) used to construct actual
//! database connections. An engine is created from a configuration instance.

use std::str::FromStr;

use crate::{configuration, Configuration, Error};

#[cfg(feature = "mysql")]
mod backend {
    pub use sqlx::mysql::{
        MySqlConnectOptions as ConnectOptions, MySqlRow as Row,
        MySqlStatement as Statement,
    };
    pub use sqlx::MySql as Database;

    #[macro_export]
    macro_rules! parameter {
        ($index:expr) => {
            "?"
        };
    }
}

#[cfg(feature = "postgres")]
mod backend {
    pub use sqlx::postgres::{
        PgConnectOptions as ConnectOptions, PgRow as Row,
        PgStatement as Statement,
    };
    pub use sqlx::Postgres as Database;

    #[macro_export]
    macro_rules! parameter {
        ($index:expre) => {
            concat!("$", stringify!($index))
        };
    }
}

#[cfg(feature = "sqlite")]
mod backend {
    pub use sqlx::sqlite::{
        SqliteConnectOptions as ConnectOptions, SqliteRow as Row,
        SqliteStatement as Statement,
    };
    pub use sqlx::Sqlite as Database;

    #[macro_export]
    macro_rules! parameter {
        ($index:expr) => {
            "?"
        };
    }
}

pub use backend::{ConnectOptions, Database, Row, Statement};

pub type Pool = sqlx::pool::Pool<Database>;
pub type Connection = sqlx::pool::PoolConnection<Database>;
pub type Transaction<'a> = sqlx::Transaction<'a, Database>;

/// An engine that produces pooled database connections.
#[derive(Debug)]
pub struct Engine {
    /// The connection pool.
    pool: Pool,
}

impl Engine {
    /// Attempts to acquire a database connection.
    pub async fn connection(&self) -> Result<Connection, Error> {
        self.pool.acquire().await
    }
}

impl From<Pool> for Engine {
    fn from(source: Pool) -> Self {
        Self { pool: source }
    }
}

impl Configuration {
    /// Constructs a database engine from this configuration.
    pub async fn engine(&self) -> Result<Engine, configuration::Error> {
        Ok(sqlx::Pool::connect_with(self.connect_options()?)
            .await?
            .into())
    }

    /// Generates database connect options.
    fn connect_options(&self) -> Result<ConnectOptions, Error> {
        FromStr::from_str(&self.connection_string)
    }
}
