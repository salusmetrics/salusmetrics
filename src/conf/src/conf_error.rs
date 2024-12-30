use thiserror::Error;

/// Represent potential error cases for configuring a Salus Metrics App
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ConfError {
    #[error("Could not derive settings from environment")]
    Env,
    #[error("Unable to properly initialize metrics db from settings")]
    MetricsDb,
    #[error("Unable to properly initialize tracing from settings")]
    Tracing,
}
