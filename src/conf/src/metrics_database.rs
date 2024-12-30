use crate::{conf_error::ConfError, settings::SharedSettings};
use clickhouse::Client;
use serde::{Deserialize, Serialize};

/// Metrics Database settings - ENV variables use the `METRICSDB` sub-prefix
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetricsDatabaseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub pass: String,
}

/// Attempt to create a Clickhouse Client for the given app_name, which will
/// be used to examine ENV for SharedSettings for this app.
pub fn try_get_metrics_client(app_name: &str) -> Result<Client, ConfError> {
    assert!(!app_name.is_empty());

    let metrics_settings = SharedSettings::try_new(app_name)?
        .metricsdb
        .ok_or(ConfError::MetricsDb)?;
    Ok(Client::default()
        .with_url(metrics_settings.url)
        .with_user(metrics_settings.user)
        .with_password(metrics_settings.pass)
        .with_database(metrics_settings.database))
}

#[cfg(test)]
mod tests {
    use crate::settings::tests::{cleanup_test_env, setup_valid_test_env, APP_NAME};

    use super::try_get_metrics_client;

    #[test]
    fn test_try_get_metrics_client() {
        // Positive test case
        setup_valid_test_env();
        try_get_metrics_client(APP_NAME).unwrap();
        cleanup_test_env();

        // Negative test case
        assert!(try_get_metrics_client(APP_NAME).is_err());
    }
}
