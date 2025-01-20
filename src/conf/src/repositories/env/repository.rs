use config::{Config, Environment};
use serde::{Deserialize, Serialize};

use crate::domain::repository::configuration_repository::{
    ConfigurationRepository, ConfigurationRepositoryError,
};

use super::env_settings::*;
use crate::domain::model::{
    compression::*, cors::*, listener::*, metrics_db::*, timeout::*, tracing::*,
};

/// `EnvRepository` provides a `ConfigurationRepository` based on the
/// application environment variables, aligning with 12 factor guidelines
/// for configuration. This functions by examining environment variables with
/// the `app_prefix` as the first portion of the variable name. The prefix
/// is then separated from variable names using `_` and each attribute in the
/// graph of names is similarly separated by`_`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnvRepository {
    layer: Option<EnvLayerSettings>,
    listener: Option<EnvListenerSettings>,
    metricsdb: Option<EnvMetricsDatabaseSettings>,
    tracing: Option<EnvTracingSettings>,
}

/// `LayerSettings` wraps the `CorsSettings` and `TimeoutSettings` into a
/// common struct which can be used to handle both in a clean manner
/// which will optionally set up a CORS layer if any is specified.
#[derive(Debug, Deserialize, Serialize, Clone)]
struct EnvLayerSettings {
    compression: Option<EnvCompressionSettings>,
    cors: Option<EnvCorsSettings>,
    timeout: Option<EnvTimeoutSettings>,
}

impl EnvRepository {
    /// Attempt to create an EnvRepository from the ENV for the specified app.
    /// app_prefix will be used as the prefix for all settings for this app.
    pub fn try_new(app_prefix: impl AsRef<str>) -> Result<Self, ConfigurationRepositoryError> {
        assert!(
            !app_prefix.as_ref().is_empty(),
            "app_prefix must be a non-empty string"
        );

        Config::builder()
            .add_source(
                Environment::with_prefix(app_prefix.as_ref())
                    .with_list_parse_key("layer.cors.origins")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()
            .map_err(|_| ConfigurationRepositoryError::Repository)?
            .try_deserialize()
            .map_err(|_| ConfigurationRepositoryError::Repository)
    }
}

impl ConfigurationRepository for EnvRepository {
    fn try_compression_settings(
        &self,
    ) -> Result<CompressionSettings, ConfigurationRepositoryError> {
        let Some(ref layer_settings) = self.layer else {
            tracing::info!("Using default HTTP Compression Layer Settings");
            return Ok(CompressionSettings::default());
        };
        let Some(ref settings) = layer_settings.compression else {
            tracing::info!("Using default HTTP Compression Layer Settings");
            return Ok(CompressionSettings::default());
        };
        Ok(settings.into())
    }

    fn try_cors_settings(&self) -> Result<CorsSettings, ConfigurationRepositoryError> {
        let Some(ref layer_settings) = self.layer else {
            tracing::error!("Missing CORS configuration in ENV");
            return Err(ConfigurationRepositoryError::Missing);
        };
        let Some(ref cors_settings) = layer_settings.cors else {
            tracing::error!("Missing CORS configuration in ENV");
            return Err(ConfigurationRepositoryError::Missing);
        };
        Ok(cors_settings.into())
    }

    fn try_listener_settings(&self) -> Result<ListenerSettings, ConfigurationRepositoryError> {
        let Some(ref listener_settings) = self.listener else {
            tracing::error!("Missing listener configuration in ENV");
            return Err(ConfigurationRepositoryError::Missing);
        };
        Ok(listener_settings.into())
    }

    fn try_metrics_db_settings(
        &self,
    ) -> Result<MetricsDatabaseSettings, ConfigurationRepositoryError> {
        let Some(ref metrics_db_settings) = self.metricsdb else {
            tracing::error!("Missing metrics db configuration in ENV");
            return Err(ConfigurationRepositoryError::Missing);
        };
        Ok(metrics_db_settings.into())
    }

    fn try_timeout_settings(&self) -> Result<TimeoutSettings, ConfigurationRepositoryError> {
        let Some(ref layer_settings) = self.layer else {
            tracing::info!("Using default timeout settings for HTTP Layers");
            return Ok(TimeoutSettings::default());
        };
        let Some(ref settings) = layer_settings.timeout else {
            tracing::info!("Using default timeout settings for HTTP Layers");
            return Ok(TimeoutSettings::default());
        };
        Ok(settings.into())
    }

    fn try_tracing_settings(&self) -> Result<TracingSettings, ConfigurationRepositoryError> {
        let Some(ref settings) = self.tracing else {
            tracing::info!("Using default tracing subscriber settings");
            return Ok(TracingSettings::default());
        };
        Ok(settings.into())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::env::{remove_var, set_var};
    use uuid::Uuid;

    use crate::domain::repository::configuration_repository::{
        ConfigurationRepository, ConfigurationRepositoryError,
    };

    use super::*;

    const VALID_SETTINGS_ARR: &[(&str, &str, &str)] = &[
        ("LAYER", "COMPRESSION_DEFLATE", "false"),
        ("LAYER", "COMPRESSION_GZIP", "true"),
        (
            "LAYER",
            "CORS_ORIGINS",
            "http://localhost:3000 http://127.0.0.1:3000",
        ),
        ("LAYER", "CORS_MAX_AGE_SECS", "60"),
        ("LAYER", "TIMEOUT_MILLIS", "4400"),
        ("LISTENER", "IPV4", "0.0.0.0"),
        ("LISTENER", "PORT", "3000"),
        ("METRICSDB", "URL", "http://localhost:8123"),
        ("METRICSDB", "DATABASE", "TEST"),
        ("METRICSDB", "USER", "TEST"),
        ("METRICSDB", "PASS", "TEST"),
        ("TRACING", "DIRECTIVE", "trace"),
    ];

    #[test]
    fn test_try_new_from_env() {
        // positive cases
        let repo = create_valid_repo();

        // Test compression
        if repo.try_compression_settings().is_err() {
            panic!("Expected compression layer to be created");
        }

        // Test timeout
        if repo.try_timeout_settings().is_err() {
            panic!("Expected timeout layer to be created");
        }

        // Test CORS
        if repo.try_cors_settings().is_err() {
            panic!("Expected valid CorsLayer");
        }

        // Test listener
        if repo.try_listener_settings().is_err() {
            panic!("Expected valid listener SocketAddr");
        }

        // Test MetricsDB
        if repo.try_metrics_db_settings().is_err() {
            panic!("Expected valid db settings");
        }

        // Test tracing - Commented out because this can only be called once
        // and is covered by an existing test in the tracing module.
        // settings.tracing.try_init_tracing_subscriber().unwrap();

        // negative cases
        let empty_repo = EnvRepository::try_new("INVALID_APP_NAME").unwrap();
        assert_eq!(
            empty_repo.try_cors_settings().unwrap_err(),
            ConfigurationRepositoryError::Missing
        );
        assert_eq!(
            empty_repo.try_listener_settings().unwrap_err(),
            ConfigurationRepositoryError::Missing
        );
        assert_eq!(
            empty_repo.try_metrics_db_settings().unwrap_err(),
            ConfigurationRepositoryError::Missing
        );
    }

    /// Self-contained method for establishing test settings
    /// and performing cleanup
    fn create_valid_repo() -> EnvRepository {
        let app_name = setup_valid_test_env();
        let repo = EnvRepository::try_new(&app_name);
        cleanup_test_env(&app_name);

        // If this didn't create a valid repo, error for the test, else return
        repo.unwrap()
    }

    /// Sets ENV variables for a valid local test. Returns
    /// String that is the APP_NAME that should be used for the test as well
    /// as what must be passed to properly clean up.
    /// Must be followed by cleanup_test_env()
    fn setup_valid_test_env() -> String {
        let app_name = Uuid::now_v7().to_string();
        for (pre, post, val) in VALID_SETTINGS_ARR {
            let key = format!("{}_{}_{}", app_name, pre, post);
            set_var(&key, val);
            println!("{} = {}", &key, val);
        }
        app_name
    }

    /// For Testing Only - cleans up ENV variables for local test
    fn cleanup_test_env(app_name: &str) {
        for (pre, post, _) in VALID_SETTINGS_ARR {
            remove_var(format!("{}_{}_{}", app_name, pre, post));
        }
    }
}
