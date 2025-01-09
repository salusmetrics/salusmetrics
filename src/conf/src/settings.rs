use crate::listener::ListenerSettings;
use crate::metrics_database::MetricsDatabaseSettings;
use crate::tracing::TracingSettings;
use crate::{conf_error::ConfError, layer::LayerSettings};
use config::{Config, Environment};
use serde::{Deserialize, Serialize};

/// Settings struct that is common between different apps that make up salus metrics
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct CommonSettings {
    layer: Option<LayerSettings>,
    listener: Option<ListenerSettings>,
    metricsdb: Option<MetricsDatabaseSettings>,
    tracing: TracingSettings,
}

impl CommonSettings {
    /// Getter for `LayerSettings`
    pub fn layer(&self) -> Option<LayerSettings> {
        self.layer.to_owned()
    }

    /// Getter for `ListenerSettings`
    pub fn listener(&self) -> Option<ListenerSettings> {
        self.listener.to_owned()
    }

    /// Getter for `MetricsDatabaseSettings`
    pub fn metricsdb(&self) -> Option<MetricsDatabaseSettings> {
        self.metricsdb.to_owned()
    }

    /// Getter for `TracingSettings`
    pub fn tracing(&self) -> TracingSettings {
        self.tracing.to_owned()
    }

    /// Attempt to create a CommonSettings from the ENV for the specified app.
    /// app_name will be used as the prefix for all settings for this app.
    pub fn try_new_from_env(app_name: impl AsRef<str>) -> Result<Self, ConfError> {
        assert!(!app_name.as_ref().is_empty());

        Config::builder()
            .add_source(
                Environment::with_prefix(app_name.as_ref())
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

/// `CommonSettingsBuilder` implements the builder pattern for `CommonSettings`
#[derive(Default)]
pub struct CommonSettingsBuilder {
    layer: Option<LayerSettings>,
    listener: Option<ListenerSettings>,
    metricsdb: Option<MetricsDatabaseSettings>,
    tracing: TracingSettings,
}

impl CommonSettingsBuilder {
    /// `CommonSettingsBuilder` constructor
    pub fn new() -> Self {
        CommonSettingsBuilder::default()
    }

    /// Set the `LayerSettings` for this builder
    pub fn layer(mut self, layer_settings: LayerSettings) -> Self {
        self.layer = Some(layer_settings);
        self
    }

    /// Set the `ListenerSettings` for this builder
    pub fn listener(mut self, listener_settings: ListenerSettings) -> Self {
        self.listener = Some(listener_settings);
        self
    }

    /// Set the `MetricsDatabaseSettings` for this builder
    pub fn metricsdb(mut self, metricsdb_settings: MetricsDatabaseSettings) -> Self {
        self.metricsdb = Some(metricsdb_settings);
        self
    }

    /// Set the `TracingSettings` for this builder
    pub fn tracing(mut self, tracing_settings: TracingSettings) -> Self {
        self.tracing = tracing_settings;
        self
    }

    pub fn build(self) -> CommonSettings {
        CommonSettings {
            layer: self.layer,
            listener: self.listener,
            metricsdb: self.metricsdb,
            tracing: self.tracing,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use clickhouse::Client;
    use std::{
        env::{remove_var, set_var},
        net::SocketAddr,
    };
    use uuid::Uuid;

    use crate::conf_error::ConfError;

    use super::CommonSettings;
    const VALID_SETTINGS_ARR: &[(&str, &str, &str)] = &[
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
        let settings = create_valid_env();

        // Test compression
        if settings
            .layer()
            .unwrap()
            .try_create_compression_layer()
            .is_none()
        {
            panic!("Expected compression layer to be created");
        }

        // Test timeout
        settings.layer().unwrap().create_timeout_layer();

        // Test CORS
        if settings
            .layer
            .to_owned()
            .unwrap()
            .try_create_cors_layer()
            .unwrap()
            .is_none()
        {
            panic!("expected valid CorsLayer");
        }

        // Test listener
        if SocketAddr::try_from(&settings.listener.to_owned().unwrap()).is_err() {
            panic!("expected valid listener SocketAddr");
        }

        // Test MetricsDB
        let _ = Client::from(&settings.metricsdb.to_owned().unwrap());

        // Test tracing - Commented out because this can only be called once
        // and is covered by an existing test in the tracing module.
        // settings.tracing.try_init_tracing_subscriber().unwrap();

        // negative case
        assert_eq!(
            CommonSettings::try_new_from_env("INVALID_APP_NAME").unwrap_err(),
            ConfError::Env
        );
    }

    /// For Testing Only - self-contained method for establishing test settings
    /// and performing cleanup
    fn create_valid_env() -> CommonSettings {
        let app_name = setup_valid_test_env();
        let settings = CommonSettings::try_new_from_env(&app_name).unwrap();
        cleanup_test_env(&app_name);

        // return for other tests to utilize
        settings
    }

    /// For Testing Only - sets ENV variables for a valid local test. Returns
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
