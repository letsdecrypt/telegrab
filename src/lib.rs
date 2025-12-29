use errors::Error;

pub mod backtrace;
pub mod configuration;
pub mod controller;
pub mod errors;
pub mod format;
pub mod graceful;
pub mod http_client;
pub mod middleware;
pub mod model;
pub mod service;
pub mod shutdown_signal;
pub mod startup;
pub mod state;
pub mod telemetry;
pub mod worker;

pub type Result<T, E = Error> = std::result::Result<T, E>;
