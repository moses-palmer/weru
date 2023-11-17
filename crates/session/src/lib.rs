use actix_session::SessionMiddleware as ActixSessionMiddleware;

pub mod configuration;
pub use configuration::Configuration;

pub mod store;
pub use store::Store;

/// The session middleware exposed by this crate.
pub type SessionMiddleware = ActixSessionMiddleware<store::Store>;

#[cfg(not(any(feature = "cookie")))]
compile_error!("At least one storage must be enabled!");
