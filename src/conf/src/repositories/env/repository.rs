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
/// application environment varialbles, aligning with 12 factor guidelines
/// for configuration.
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
pub struct EnvLayerSettings {
    compression: Option<EnvCompressionSettings>,
    cors: Option<EnvCorsSettings>,
    timeout: Option<EnvTimeoutSettings>,
}

impl EnvRepository {
    /// Attempt to create an EnvRepository from the ENV for the specified app.
    /// app_name will be used as the prefix for all settings for this app.
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
