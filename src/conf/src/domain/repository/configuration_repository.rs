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

/// `ConfigurationRepository` represents the logical operations that must be
/// available for all structs that will provide access to the underlying
/// configuration settings.
pub trait ConfigurationRepository: 'static + Clone + Send + Sync {
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

#[cfg(test)]
pub(crate) mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[derive(Clone, Default, Debug)]
    pub(crate) struct MockConfigurationRepository {
        compression_result: Option<Result<CompressionSettings, ConfigurationRepositoryError>>,
        cors_result: Option<Result<CorsSettings, ConfigurationRepositoryError>>,
        listener_result: Option<Result<ListenerSettings, ConfigurationRepositoryError>>,
        metrics_db_result: Option<Result<MetricsDatabaseSettings, ConfigurationRepositoryError>>,
        timeout_result: Option<Result<TimeoutSettings, ConfigurationRepositoryError>>,
        tracing_result: Option<Result<TracingSettings, ConfigurationRepositoryError>>,
    }

    impl MockConfigurationRepository {
        pub(crate) fn set_compression_result(
            &mut self,
            compression: Result<CompressionSettings, ConfigurationRepositoryError>,
        ) {
            self.compression_result = Some(compression)
        }

        pub(crate) fn set_cors_result(
            &mut self,
            cors: Result<CorsSettings, ConfigurationRepositoryError>,
        ) {
            self.cors_result = Some(cors)
        }

        pub(crate) fn set_listener_result(
            &mut self,
            listener: Result<ListenerSettings, ConfigurationRepositoryError>,
        ) {
            self.listener_result = Some(listener)
        }

        pub(crate) fn set_metrics_db(
            &mut self,
            metrics_db: Result<MetricsDatabaseSettings, ConfigurationRepositoryError>,
        ) {
            self.metrics_db_result = Some(metrics_db)
        }

        pub(crate) fn set_timeout_result(
            &mut self,
            timeout: Result<TimeoutSettings, ConfigurationRepositoryError>,
        ) {
            self.timeout_result = Some(timeout)
        }

        pub(crate) fn set_tracing_result(
            &mut self,
            tracing: Result<TracingSettings, ConfigurationRepositoryError>,
        ) {
            self.tracing_result = Some(tracing)
        }
    }

    impl ConfigurationRepository for MockConfigurationRepository {
        fn try_compression_settings(
            &self,
        ) -> Result<CompressionSettings, ConfigurationRepositoryError> {
            self.compression_result.to_owned().unwrap()
        }

        fn try_cors_settings(&self) -> Result<CorsSettings, ConfigurationRepositoryError> {
            self.cors_result.to_owned().unwrap()
        }

        fn try_listener_settings(&self) -> Result<ListenerSettings, ConfigurationRepositoryError> {
            self.listener_result.to_owned().unwrap()
        }

        fn try_metrics_db_settings(
            &self,
        ) -> Result<MetricsDatabaseSettings, ConfigurationRepositoryError> {
            self.metrics_db_result.to_owned().unwrap()
        }

        fn try_timeout_settings(&self) -> Result<TimeoutSettings, ConfigurationRepositoryError> {
            self.timeout_result.to_owned().unwrap()
        }

        fn try_tracing_settings(&self) -> Result<TracingSettings, ConfigurationRepositoryError> {
            self.tracing_result.to_owned().unwrap()
        }
    }

    #[test]
    fn test_mock_repository() {
        let mut repo = MockConfigurationRepository::default();

        // Set each response we want
        repo.set_compression_result(Ok(CompressionSettings {
            gzip: Some(true),
            deflate: Some(false),
        }));
        repo.set_cors_result(Ok(CorsSettings {
            max_age_secs: Some(10),
            origins: vec!["test.com".to_owned()],
        }));
        repo.set_listener_result(Ok(ListenerSettings {
            port: 9000,
            ipv4: Some(Ipv4Addr::LOCALHOST),
            ipv6: None,
        }));
        repo.set_metrics_db(Ok(MetricsDatabaseSettings::new(
            "http://localhost:7777",
            "METRICS",
            "username",
            "password",
        )));
        repo.set_timeout_result(Ok(TimeoutSettings { millis: 15000 }));
        repo.set_tracing_result(Ok(TracingSettings {
            directive: "trace".to_owned(),
        }));

        // Test each method of the mock repo
        assert!(
            repo.try_compression_settings().is_ok(),
            "Expected result for compression settings"
        );

        assert!(
            repo.try_cors_settings().is_ok(),
            "Expected result for CORS settings"
        );

        assert!(
            repo.try_listener_settings().is_ok(),
            "Expected result for listener settings"
        );

        assert!(
            repo.try_metrics_db_settings().is_ok(),
            "Expected result for metrics db settings"
        );

        assert!(
            repo.try_timeout_settings().is_ok(),
            "Expected result for timeout settings"
        );

        assert!(
            repo.try_tracing_settings().is_ok(),
            "Expected result for tracing settings"
        );
    }
}
