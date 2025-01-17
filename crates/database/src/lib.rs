pub mod configuration;
pub use configuration::Configuration;

pub mod engine;
pub use engine::{Connection, Database, Engine, Row, Statement, Transaction};

pub mod error;
pub use error::Error;

pub mod traits;
pub use traits::Entity;

pub use sqlx;

#[cfg(not(any(
    feature = "mysql",
    feature = "postgres",
    feature = "sqlite",
)))]
compile_error!("At least one backend must be enabled!");
