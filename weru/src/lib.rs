// Expose the framework
pub use actix_rt::main;
pub mod actix {
    pub use actix::*;
    pub use actix_http as http;
    pub use actix_rt as rt;
    pub use actix_web as web;
}
