use thiserror::Error;

use crate::domain::model::{
    compression::CompressionSettings, configuration_error::ConfigurationError, cors::CorsSettings,
    listener::ListenerSettings, metrics_db::MetricsDatabaseSettings, timeout::TimeoutSettings,
    tracing::TracingSettings,
};

/// `ConfigurationRepositoryError` represents the domain errors that can arise
/// when calling a given `ConfigurationService`
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ConfigurationRepositoryError {
    /// `Missing` indicates that the repository could not find configuration
    /// for this particular setting
    #[error("Missing configuration for this setting")]
    Missing,
    /// `Model` indicates that something errored in the process of
    /// interacting with the domain model objects which underpin the repository
    #[error("Domain model error")]
    Model(#[from] ConfigurationError),
    /// `Repository` error indicates that the error for this configuration
    /// arose from the underlying repo
    #[error("Repository error")]
    Repository,
}

pub trait ConfigurationRepository {
    /// `try_compression_settings` attempts fetch `CompressionSettings`
    fn try_compression_settings(&self)
        -> Result<CompressionSettings, ConfigurationRepositoryError>;

    /// `try_cors_settings` attempts to fetch `CorsSettings`
    fn try_cors_settings(&self) -> Result<CorsSettings, ConfigurationRepositoryError>;

    /// `try_timeout_settings` attempts to fetch `TimeoutSettings`
    fn try_timeout_settings(&self) -> Result<TimeoutSettings, ConfigurationRepositoryError>;

    /// `try_metrics_db_settings` attempts to fetch `MetricsDatabaseSettings`
    fn try_metrics_db_settings(
        &self,
    ) -> Result<MetricsDatabaseSettings, ConfigurationRepositoryError>;

    /// `try_listener_settings` attempts to fetch `ListenerSettings`
    fn try_listener_settings(&self) -> Result<ListenerSettings, ConfigurationRepositoryError>;

    /// `try_tracing_settings` attempts to fetch `TracingSettings`
    fn try_tracing_settings(&self) -> Result<TracingSettings, ConfigurationRepositoryError>;
}
