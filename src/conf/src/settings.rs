use crate::conf_error::ConfError;
use crate::metrics_database::MetricsDatabaseSettings;
use crate::tracing::TracingSettings;
use config::{Config, Environment};
use serde::{Deserialize, Serialize};

/// Settings struct that is common between different apps that make up salus metrics
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SharedSettings {
    pub metricsdb: Option<MetricsDatabaseSettings>,
    pub tracing: TracingSettings,
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

#[cfg(test)]
pub(crate) mod tests {
    use std::env::{remove_var, set_var};

    use crate::conf_error::ConfError;

    use super::SharedSettings;
    pub(crate) const APP_NAME: &str = "TEST_SM";
    const VALID_SETTINGS_ARR: &[(&str, &str, &str)] = &[
        ("TRACING", "DIRECTIVE", "warn"),
        ("METRICSDB", "URL", "http://localhost:8123"),
        ("METRICSDB", "DATABASE", "TEST"),
        ("METRICSDB", "USER", "TEST"),
        ("METRICSDB", "PASS", "TEST"),
    ];

    #[test]
    fn test_try_new() {
        // positive case
        get_valid_env();
        // negative case
        assert_eq!(
            SharedSettings::try_new(APP_NAME).unwrap_err(),
            ConfError::Env
        );
    }

    /// For Testing Only - self-contained method for establishing test settings
    /// and performing cleanup
    pub(crate) fn get_valid_env() -> SharedSettings {
        setup_valid_test_env();
        let settings = SharedSettings::try_new(APP_NAME).unwrap();
        cleanup_test_env();

        // Provide for other tests to utilize
        settings
    }

    /// For Testing Only - sets ENV variables for a valid local test
    /// Must be followed by cleanup_test_env()
    pub(crate) fn setup_valid_test_env() {
        for (pre, post, val) in VALID_SETTINGS_ARR {
            set_var(format!("{}_{}_{}", APP_NAME, pre, post), val);
        }
    }

    /// For Testing Only - cleans up ENV variables for local test
    pub(crate) fn cleanup_test_env() {
        for (pre, post, _) in VALID_SETTINGS_ARR {
            remove_var(format!("{}_{}_{}", APP_NAME, pre, post));
        }
    }
}
