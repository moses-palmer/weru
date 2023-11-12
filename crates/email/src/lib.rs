pub mod configuration;
pub use configuration::Configuration;

pub mod engine;
pub use engine::Engine;

mod error;
pub use error::Error;

mod traits;
pub use traits::*;

pub mod sender;
pub mod template;

pub use lettre;

#[cfg(not(any(feature = "drop")))]
compile_error!("At least one backend must be enabled!");
