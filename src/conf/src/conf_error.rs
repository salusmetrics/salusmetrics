use thiserror::Error;

/// Represent potential error cases for configuring a Salus Metrics App
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ConfError {
    #[error("Could not derive settings from environment")]
    Env,
    #[error("Could not properly determine address or port to listen on")]
    Listener,
    #[error("Unable to properly initialize metrics db from settings")]
    MetricsDb,
    #[error("Unable to properly initialize tracing from settings")]
    Tracing,
}
