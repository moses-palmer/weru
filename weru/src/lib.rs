#[cfg(feature = "cache")]
pub mod cache {
    //! # The *weru* cache
    //!
    //! This module provides a key-value store, or a
    //! [cache](weru_cache::Cache).
    //!
    //! A cache provides a mapping from a key to a value, with a specified
    //! time-to-live, after which the value will be unavailable. A cache is
    //! constructed by an [engine](weru_cache::engine::Engine).
    //!
    //! # Examples
    //!
    //! ```
    //! # use std::time::Duration;
    //! # use weru_cache::{
    //!        Cache,
    //!        Configuration,
    //!        Engine,
    //!        engine::backends::local,
    //! };
    //! # actix_rt::Runtime::new().unwrap().block_on(async {
    //!
    //! // Create a configuration for a process local cache.
    //! //
    //! // You would normally load this value from a file.
    //! let configuration = Configuration::Local(local::Configuration);
    //!
    //! // Create a cache engine from the configuration...
    //! let engine = configuration.engine().await.unwrap();
    //!
    //! // ...and then create a cache from the engine
    //! let cache = engine.cache::<String, String>("cache-name").await.unwrap();
    //!
    //! let value = "value".into();
    //! cache.put("key".to_string(), value, Duration::from_secs(10)).await
    //!        .unwrap();
    //!
    //! assert_eq!(
    //!        Some("value".into()),
    //!        cache.get(&"key".to_string()).await.unwrap(),
    //! );
    //! # });
    //! ```
    pub use weru_cache::*;
}

#[cfg(feature = "channel")]
pub mod channel {
    //! # The *weru* channel
    //!
    //! This module provides a channel, a means of sending and listening for
    //! events on a topic.
    //!
    //! # Examples
    //!
    //! ```
    //! # use std::time::Duration;
    //! # use weru_channel::{
    //!        Channel,
    //!        Configuration,
    //!        Engine,
    //!        engine::backends::local,
    //! };
    //! # actix_rt::Runtime::new().unwrap().block_on(async {
    //! use futures::StreamExt;
    //!
    //! // Create a configuration for a process local channel.
    //! //
    //! // You would normally load this value from a file.
    //! let configuration = Configuration::Local(local::Configuration {
    //!        queue_size: 10,
    //! });
    //!
    //! // Create a channel engine from the configuration...
    //! let engine = configuration.engine().await.unwrap();
    //!
    //! // ...and then create a channel and a listener from the engine
    //! let channel = engine.channel::<String>("topic".to_string()).await
    //!        .unwrap();
    //! let listener = engine.channel::<String>("topic".to_string()).await
    //!        .unwrap().listen().await.unwrap();
    //!
    //! let event = "event".into();
    //! channel.broadcast(event).await.unwrap();
    //!
    //! assert_eq!(
    //!        vec!["event".to_string()],
    //!        listener.take(1).map(Result::unwrap).collect::<Vec<_>>().await,
    //! );
    //! # });
    //! ```
    pub use weru_channel::*;
}

#[cfg(feature = "database")]
pub mod database {
    //! # The *weru* database
    //!
    //! # Examples
    //!
    //! ```
    //! # use std::time::Duration;
    //! # use weru_database::{Configuration, Engine};
    //! # actix_rt::Runtime::new().unwrap().block_on(async {
    //! use weru_database::sqlx::prelude::*;
    //!
    //! // Create a configuration for a Sqlite in-memory database.
    //! //
    //! // You would normally load this value from a file.
    //! let configuration = Configuration {
    //!        connection_string: "sqlite::memory:".into(),
    //! };
    //!
    //! // Create a database engine from the configuration...
    //! let engine = configuration.engine().await.unwrap();
    //!
    //! // ...and then create a connection from the engine...
    //! let mut connection = engine.connection().await.unwrap();
    //! {
    //!        // ...and finally create a transaction to interact with the
    //!        // database
    //!        let mut tx = connection.begin().await.unwrap();
    //!        tx.execute(r#"
    //!            CREATE TABLE Test (
    //!                value TEXT NOT NULL
    //!            );
    //!            INSERT INTO Test(value)
    //!            VALUES("value");
    //!        "#).await.unwrap();
    //!        tx.commit().await.unwrap();
    //! }
    //!
    //! let value: String = {
    //!        let mut tx = connection.begin().await.unwrap();
    //!        tx.fetch_one(r#"
    //!            SELECT value from Test
    //!        "#).await.unwrap().get(0)
    //! };
    //! assert_eq!(value, String::from("value"));
    //! # });
    //! ```
    pub use weru_database::*;
}

// Expose the framework
pub use actix_rt::main;
pub mod actix {
    pub use actix::*;
    pub use actix_http as http;
    pub use actix_rt as rt;
    pub use actix_web as web;
}
