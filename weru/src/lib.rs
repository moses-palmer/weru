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

// Expose the framework
pub use actix_rt::main;
pub mod actix {
    pub use actix::*;
    pub use actix_http as http;
    pub use actix_rt as rt;
    pub use actix_web as web;
}
