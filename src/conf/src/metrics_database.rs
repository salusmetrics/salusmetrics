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
pub fn try_create_metrics_client(app_name: &str) -> Result<Client, ConfError> {
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
    use crate::settings::tests::{cleanup_test_env, setup_valid_test_env};

    use super::try_create_metrics_client;

    #[test]
    fn test_try_create_metrics_client() {
        // Positive test case
        let app_name = setup_valid_test_env();
        try_create_metrics_client(&app_name).unwrap();
        cleanup_test_env(&app_name);

        // Negative test case
        assert!(try_create_metrics_client("INVALID_APP_NAME").is_err());
    }
}
