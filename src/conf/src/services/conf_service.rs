use crate::domain::{
    model::configuration_error::ConfigurationError,
    repository::configuration_repository::{ConfigurationRepository, ConfigurationRepositoryError},
    service::configuration_service::{ConfigurationService, ConfigurationServiceError},
};

/// `ConfService<T>` is a generic implementation of the `ConfigurationService`
/// that can use any corresponding `ConfigurationRepository` to provide
/// configuration to apps
#[derive(Clone, Debug)]
pub struct ConfService<T>
where
    T: ConfigurationRepository + std::fmt::Debug,
{
    pub(crate) conf_repository: T,
}

impl<T> ConfigurationService for ConfService<T>
where
    T: ConfigurationRepository + std::fmt::Debug,
{
    fn try_compression_layer(
        &self,
    ) -> Result<tower_http::compression::CompressionLayer, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_compression_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    fn try_cors_layer(&self) -> Result<tower_http::cors::CorsLayer, ConfigurationServiceError> {
        (&self
            .conf_repository
            .try_cors_settings()
            .map_err(map_repo_err_to_service_err)?)
            .try_into()
            .map_err(map_configuration_err_to_service_err)
    }

    fn try_listener_socket_addr(&self) -> Result<std::net::SocketAddr, ConfigurationServiceError> {
        (&self
            .conf_repository
            .try_listener_settings()
            .map_err(map_repo_err_to_service_err)?)
            .try_into()
            .map_err(map_configuration_err_to_service_err)
    }

    fn try_metrics_db_client(&self) -> Result<clickhouse::Client, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_metrics_db_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    fn try_timeout_layer(
        &self,
    ) -> Result<tower_http::timeout::TimeoutLayer, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_timeout_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    fn try_tracing_subscriber_setup(&self) -> Result<(), ConfigurationServiceError> {
        self.conf_repository
            .try_tracing_settings()
            .map_err(map_repo_err_to_service_err)?
            .try_init_tracing_subscriber()
            .map_err(map_configuration_err_to_service_err)
    }
}

/// `map_configuration_err_to_service_err` is a simple function that is used to
/// map between the domain model errors into the outermost service errors that
/// are delivered to consumers of this service
fn map_configuration_err_to_service_err(conf_err: ConfigurationError) -> ConfigurationServiceError {
    match conf_err {
        ConfigurationError::Invalid => ConfigurationServiceError::Invalid,
        ConfigurationError::Parse => ConfigurationServiceError::Invalid,
    }
}

/// `map_repo_err_to_service_err` is a simple function that is used to map
/// between errors that arise at the repository level into errors that can be
/// delivered to consumers of the service layer.
fn map_repo_err_to_service_err(
    repo_err: ConfigurationRepositoryError,
) -> ConfigurationServiceError {
    match repo_err {
        ConfigurationRepositoryError::Missing => ConfigurationServiceError::Missing,
        ConfigurationRepositoryError::Model(_) => ConfigurationServiceError::Invalid,
        ConfigurationRepositoryError::Repository => ConfigurationServiceError::Repository,
    }
}

impl<T> ConfService<T>
where
    T: ConfigurationRepository + std::fmt::Debug,
{
    /// Simple constructor for  `ConfService`
    pub fn new(conf_repo: T) -> Self {
        Self {
            conf_repository: conf_repo,
        }
    }
}
