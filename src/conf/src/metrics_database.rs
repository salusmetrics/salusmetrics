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
