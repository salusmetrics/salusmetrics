use thiserror::Error;

/// `ConfError` represents potential error cases for configuring a metrics
/// application instance. Errors of type ConfError should only be encountered
/// at application startup or in the event where configuration is somehow
/// reloaded.
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ConfError {
    #[error("Unable to properly set up CORS layer from env configuration")]
    Cors,
    #[error("Could not derive settings from environment")]
    Env,
    #[error("Could not properly determine layer settings from env")]
    Layer,
    #[error("Could not properly determine address or port to listen on")]
    Listener,
    #[error("Unable to properly initialize metrics db from settings")]
    MetricsDb,
    #[error("Unable to properly initialize tracing from settings")]
    Tracing,
}
