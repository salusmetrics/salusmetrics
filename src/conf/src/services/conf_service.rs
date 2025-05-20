use tracing::instrument;

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
    #[instrument]
    fn try_compression_layer(
        &self,
    ) -> Result<tower_http::compression::CompressionLayer, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_compression_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    #[instrument]
    fn try_cors_layer(&self) -> Result<tower_http::cors::CorsLayer, ConfigurationServiceError> {
        (&self
            .conf_repository
            .try_cors_settings()
            .map_err(map_repo_err_to_service_err)?)
            .try_into()
            .map_err(map_configuration_err_to_service_err)
    }

    #[instrument]
    fn try_ip_source(&self) -> Result<axum_client_ip::ClientIpSource, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_ip_source_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    #[instrument]
    fn try_listener_socket_addr(&self) -> Result<std::net::SocketAddr, ConfigurationServiceError> {
        (&self
            .conf_repository
            .try_listener_settings()
            .map_err(map_repo_err_to_service_err)?)
            .try_into()
            .map_err(map_configuration_err_to_service_err)
    }

    #[instrument]
    fn try_metrics_db_client(&self) -> Result<clickhouse::Client, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_metrics_db_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    #[instrument]
    fn try_timeout_layer(
        &self,
    ) -> Result<tower_http::timeout::TimeoutLayer, ConfigurationServiceError> {
        Ok((&self
            .conf_repository
            .try_timeout_settings()
            .map_err(map_repo_err_to_service_err)?)
            .into())
    }

    #[instrument]
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
    #[instrument]
    pub fn new(conf_repo: T) -> Self {
        Self {
            conf_repository: conf_repo,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    use crate::domain::model::compression::CompressionSettings;
    use crate::domain::model::cors::CorsSettings;
    use crate::domain::model::ip_source::IpSourceSettings;
    use crate::domain::model::listener::ListenerSettings;
    use crate::domain::model::metrics_db::MetricsDatabaseSettings;
    use crate::domain::model::timeout::TimeoutSettings;
    use crate::domain::model::tracing::TracingSettings;
    use crate::domain::repository::configuration_repository::tests::MockConfigurationRepository;
    use crate::domain::service::configuration_service::ConfigurationService;

    #[test]
    fn test_conf_service() {
        // Positive test cases
        let mut test_success_repo = MockConfigurationRepository::default();

        test_success_repo.set_compression_result(Ok(CompressionSettings::default()));
        test_success_repo.set_cors_result(Ok(CorsSettings {
            max_age_secs: Some(20),
            origins: vec!["test.com".to_owned()],
        }));
        test_success_repo.set_ip_source_result(Ok(IpSourceSettings::default()));
        test_success_repo.set_listener_result(Ok(ListenerSettings {
            port: 8444,
            ipv4: Some(Ipv4Addr::LOCALHOST),
            ipv6: None,
        }));
        test_success_repo.set_metrics_db(Ok(MetricsDatabaseSettings::new(
            "http://localhost:3344",
            "METRICS_DB",
            "user",
            "pass",
        )));
        test_success_repo.set_timeout_result(Ok(TimeoutSettings { millis: 5599 }));
        test_success_repo.set_tracing_result(Ok(TracingSettings {
            directive: "trace".to_owned(),
        }));

        let test_success_service = ConfService::new(test_success_repo);
        assert!(
            test_success_service.try_compression_layer().is_ok(),
            "Expected to create valid compression layer"
        );

        assert!(
            test_success_service.try_cors_layer().is_ok(),
            "Expected to create valid CORS layer"
        );

        assert!(
            test_success_service.try_ip_source().is_ok(),
            "Expected a valid ClientIpSource"
        );

        assert!(
            test_success_service.try_listener_socket_addr().is_ok(),
            "Expected to create valid listener soccet address"
        );

        assert!(
            test_success_service.try_metrics_db_client().is_ok(),
            "Expected to create valid metrics db client"
        );

        assert!(
            test_success_service.try_timeout_layer().is_ok(),
            "Expected to create valid timeout layer"
        );

        // The following test is commented out because multiple calls to
        // set up a tracing subscriber will cause a panic.
        //       assert!(
        //           test_service.try_tracing_subscriber_setup().is_ok(),
        //           "Expected to create valid compression layer"
        //       );

        // Negative test cases
        let mut test_failure_repo = MockConfigurationRepository::default();

        test_failure_repo.set_compression_result(Err(ConfigurationRepositoryError::Repository));
        test_failure_repo.set_cors_result(Err(ConfigurationRepositoryError::Missing));
        test_failure_repo.set_listener_result(Err(ConfigurationRepositoryError::Missing));
        test_failure_repo.set_metrics_db(Err(ConfigurationRepositoryError::Missing));
        test_failure_repo.set_timeout_result(Err(ConfigurationRepositoryError::Repository));
        test_failure_repo.set_tracing_result(Err(ConfigurationRepositoryError::Repository));

        let test_failure_service = ConfService::new(test_failure_repo);
        assert!(
            test_failure_service.try_compression_layer().is_err(),
            "Expected error for compression layer"
        );

        assert!(
            test_failure_service.try_cors_layer().is_err(),
            "Expected error for CORS layer"
        );

        assert!(
            test_failure_service.try_listener_socket_addr().is_err(),
            "Expected error for listener soccet address"
        );

        assert!(
            test_failure_service.try_metrics_db_client().is_err(),
            "Expected error for metrics db client"
        );

        assert!(
            test_failure_service.try_timeout_layer().is_err(),
            "Expected error for timeout layer"
        );

        assert!(
            test_failure_service.try_tracing_subscriber_setup().is_err(),
            "Expected error for tracing settings"
        );
    }
}
