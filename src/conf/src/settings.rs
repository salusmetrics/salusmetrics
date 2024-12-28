use crate::conf_error::ConfError;
use crate::metrics_database::MetricsDatabaseSettings;
use crate::tracing::TracingSettings;
use config::{Config, Environment};
use serde::{Deserialize, Serialize};

/// Settings struct that is common between different apps that make up salus metrics
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SharedSettings {
    pub tracing: TracingSettings,
    pub metricsdb: Option<MetricsDatabaseSettings>,
}

impl SharedSettings {
    /// Attempt to create a SharedSettings from the ENV for the specified app.
    /// app_name will be used as the prefix for all settings for this app.
    pub fn try_new(app_name: &str) -> Result<Self, ConfError> {
        assert!(!app_name.is_empty());

        let config = Config::builder()
            .add_source(Environment::with_prefix(app_name).separator("_"))
            .build()
            .map_err(|_| ConfError::Env)?;
        let setting: SharedSettings = config.try_deserialize().map_err(|_| ConfError::Env)?;
        Ok(setting)
    }
}
