use errors::Error;

pub mod backtrace;
pub mod configuration;
pub mod controller;
pub mod errors;
pub mod format;
pub use telegrab_core::graceful;
pub use telegrab_core::http_client;
pub mod listener;
pub mod middleware;
pub mod model;
pub mod repository;
pub mod schema;
pub use telegrab_core::service;
pub mod shutdown_signal;
pub mod startup;
pub mod state;
pub mod telemetry;
pub mod worker;

pub type Result<T, E = Error> = std::result::Result<T, E>;
