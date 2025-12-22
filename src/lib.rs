use errors::Error;

pub mod backtrace;
pub mod configuration;
pub mod controller;
pub mod errors;
pub mod grab_worker;
pub mod middleware;
pub mod shutdown_signal;
pub mod startup;
pub mod telemetry;

pub type Result<T, E = Error> = std::result::Result<T, E>;