//! Run a Symfony application using Claviron's PHP engine and runner shell.
//!
//! Applications provide their kernel entry and configuration. Claviron owns
//! the engine, admission, Orbit transport, workers and graceful shutdown.

mod application;
pub mod options;
mod runner;

pub use options::Options;
pub use runner::Runner;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
