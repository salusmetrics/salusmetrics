use crate::listener::ListenerSettings;
use crate::metrics_database::MetricsDatabaseSettings;
use crate::tracing::TracingSettings;
use crate::{conf_error::ConfError, layer::LayerSettings};
use config::{Config, Environment};
use serde::{Deserialize, Serialize};

/// Settings struct that is common between different apps that make up salus metrics
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SharedSettings {
    pub layer: Option<LayerSettings>,
    pub listener: Option<ListenerSettings>,
    pub metricsdb: Option<MetricsDatabaseSettings>,
    pub tracing: TracingSettings,
}

impl SharedSettings {
    /// Attempt to create a SharedSettings from the ENV for the specified app.
    /// app_name will be used as the prefix for all settings for this app.
    pub fn try_new(app_name: &str) -> Result<Self, ConfError> {
        assert!(!app_name.is_empty());

        Config::builder()
            .add_source(
                Environment::with_prefix(app_name)
                    .with_list_parse_key("layer.cors.origins")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()
            .map_err(|_| ConfError::Env)?
            .try_deserialize()
            .map_err(|_| ConfError::Env)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::env::{remove_var, set_var};
    use uuid::Uuid;

    use crate::conf_error::ConfError;

    use super::SharedSettings;
    const VALID_SETTINGS_ARR: &[(&str, &str, &str)] = &[
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
        ("TRACING", "DIRECTIVE", "warn"),
    ];

    #[test]
    fn test_try_new() {
        // positive case
        create_valid_env();
        // negative case
        assert_eq!(
            SharedSettings::try_new("INVALID_APP_NAME").unwrap_err(),
            ConfError::Env
        );
    }

    /// For Testing Only - self-contained method for establishing test settings
    /// and performing cleanup
    pub(crate) fn create_valid_env() -> SharedSettings {
        let app_name = setup_valid_test_env();
        let settings = SharedSettings::try_new(&app_name).unwrap();
        cleanup_test_env(&app_name);

        // return for other tests to utilize
        settings
    }

    /// For Testing Only - sets ENV variables for a valid local test. Returns
    /// String that is the APP_NAME that should be used for the test as well
    /// as what must be passed to properly clean up.
    /// Must be followed by cleanup_test_env()
    pub(crate) fn setup_valid_test_env() -> String {
        let app_name = Uuid::now_v7().to_string();
        for (pre, post, val) in VALID_SETTINGS_ARR {
            let key = format!("{}_{}_{}", app_name, pre, post);
            set_var(&key, val);
            println!("{} = {}", &key, val);
        }
        app_name
    }

    /// For Testing Only - cleans up ENV variables for local test
    pub(crate) fn cleanup_test_env(app_name: &str) {
        for (pre, post, _) in VALID_SETTINGS_ARR {
            remove_var(format!("{}_{}_{}", app_name, pre, post));
        }
    }
}
