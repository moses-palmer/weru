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

    /// Defines database entities.
    ///
    /// This macro allows you to specify a `struct` that corresponds to a
    /// table. The macro argument specifies the name of the table, and the
    /// `struct` fields specify the columns. The first field is the unique
    /// primary key.
    ///
    /// Please see the trait [`Entity`](weru_database::Entity) for more
    /// information.
    ///
    /// # Examples
    ///
    /// ```
    /// # use weru::database::entity;
    /// # use std::time::Duration;
    /// # use weru_database::{Configuration, Engine, Entity};
    /// # use weru_database::sqlx::prelude::*;
    /// # actix_rt::Runtime::new().unwrap().block_on(async {
    /// # let engine = Configuration {
    /// #     connection_string: "sqlite::memory:".into(),
    /// # }.engine().await.unwrap();
    ///
    /// #[entity(Pets)]
    /// #[derive(Debug, PartialEq)]
    /// pub struct Pet {
    ///     pub name: String,
    ///     pub leg_count: u8,
    ///     pub pettable: bool,
    /// }
    ///
    /// let mut connection = engine.connection().await.unwrap();
    /// {
    ///     let mut tx = connection.begin().await.unwrap();
    /// #    tx.execute(r#"
    /// #        CREATE TABLE Pets (
    /// #            name TEXT NOT NULL,
    /// #            leg_count INT NOT NULL,
    /// #            pettable BOOLEAN NOT NULL
    /// #        );
    /// #    "#).await.unwrap();
    ///     let description = PetDescription {
    ///         leg_count: Some(8),
    ///         pettable: Some(false),
    ///     };
    ///     let pet = description.entity("Spidey".into()).unwrap();
    ///     pet.create(&mut *tx).await.unwrap();
    ///     let recreated = Pet::read(&mut *tx, &"Spidey".into()).await
    ///         .unwrap()
    ///         .unwrap();
    ///     assert_eq!(pet, recreated);
    ///     let new_pet = Pet {
    ///         pettable: true,
    ///         ..pet
    ///     };
    ///     new_pet.update(&mut *tx).await.unwrap();
    ///     let recreated = Pet::read(&mut *tx, &"Spidey".into()).await
    ///         .unwrap()
    ///         .unwrap();
    ///     assert_eq!(new_pet, recreated);
    /// }
    /// # });
    /// ```
    pub use weru_macros::database_entity as entity;
}

#[cfg(feature = "email")]
pub mod email {
    //! # The *weru* email sender
    //!
    //! # Examples
    //!
    //! ```
    //! # use std::time::Duration;
    //! # use weru_email::{Configuration, Engine, configuration, sender};
    //! # use weru_email::engine::backends::drop;
    //! # use weru_email::configuration::Transport;
    //! # actix_rt::Runtime::new().unwrap().block_on(async {
    //!
    //! // Create a configuration for a no-op email sender.
    //! //
    //! // You would normally load this value from a file.
    //! let configuration = Configuration {
    //!     from: "Test Sender <test@email.test>".parse().unwrap(),
    //!     templates: configuration::Templates {
    //!         default_language: "l1".into(),
    //!         path: "../crates/email/resources/test/email/template/valid.toml"
    //!             .into(),
    //!     },
    //!     transport: Transport::Drop(drop::Configuration),
    //! };
    //!
    //! // Create an e-mail engine from the configuration...
    //! let engine = configuration.engine().await.unwrap();
    //!
    //! // ...and then create a sender from the engine...
    //! let sender = engine.sender().await;
    //!
    //! let replacements = [
    //!     ("replace".into(), "Hello!".into()),
    //! ].iter().cloned().collect();
    //! sender.send(
    //!     sender::Mailboxes::new()
    //!         .with("Recipient 1 <recipient1@email.test>".parse().unwrap())
    //!         .with("Recipient 2 <recipient2@email.test>".parse().unwrap())
    //!         .with("Recipient 3 <recipient3@email.test>".parse().unwrap()),
    //!     &["en-GB".into()],
    //!     &"t1".into(),
    //!     &replacements,
    //! ).await.unwrap();
    //! # });
    //! ```
    pub use weru_email::*;
}

#[cfg(feature = "session")]
pub mod session {
    //! # The *weru* session middleware.
    //!
    //! # Examples
    //!
    //! ```
    //! # use weru_session::Configuration;
    //! # use weru_session::store::cookie;
    //! # actix_rt::Runtime::new().unwrap().block_on(async {
    //!
    //! // Create a configuration for a cookie based session middleware
    //! //
    //! // You would normally load this value from a file.
    //! let configuration = Configuration::Cookie( cookie::Configuration {
    //!    secret: "\
    //!        e464c40145589aad0a25331fdc835dbd1a442e53b1604c1bf1665671a478324d\
    //!        f42ca55f76ef4f5c5e25e6ca18438566ca6fff5cefcc83a0042157df9dee4521"
    //!        .parse().unwrap(),
    //!    name: "cookie-name".into(),
    //! });
    //!
    //! // Create the cookie store
    //! let store = configuration.store().await.unwrap();
    //!
    //! // Create the session middleware
    //! let middleware = store.clone().middleware(&configuration);
    //! # });
    //! ```
    pub use weru_session::*;
}

// Expose the framework
pub use actix_rt::main;
pub mod actix {
    pub use actix::*;
    pub use actix_http as http;
    pub use actix_rt as rt;
    pub use actix_session as session;
    pub use actix_web as web;
    pub use actix_web_actors as web_actors;
}

// Expose libraries
pub use async_trait;
pub use env_logger;
pub use futures;
pub use log;
pub use thiserror;
pub use toml;
