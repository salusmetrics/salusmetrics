use clickhouse::Client;

use crate::conf_error::ConfError;
use crate::settings::CommonSettings;

/// `CommonAppState` implements `Clone` and is intended to be used as a basis
/// for axum state across metrics applications. Provides Clickhouse `Client`
/// that will allow a common connection pool to be used for all http requests.
#[derive(Clone)]
pub struct CommonAppState {
    pub metrics_db_client: Client,
}

impl TryFrom<&CommonSettings> for CommonAppState {
    type Error = ConfError;
    fn try_from(value: &CommonSettings) -> Result<Self, Self::Error> {
        let metrics_settings = value.metricsdb.to_owned().ok_or(ConfError::MetricsDb)?;

        Ok(CommonAppState {
            metrics_db_client: Client::from(&metrics_settings),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{metrics_database::MetricsDatabaseSettings, settings::CommonSettings};

    use super::CommonAppState;

    #[test]
    fn test_try_from() {
        //positive test case
        let valid_settings = CommonSettings {
            metricsdb: Some(MetricsDatabaseSettings {
                database: "METRICS_DB".to_owned(),
                pass: "pass".to_owned(),
                url: "http://localhost:8123".to_owned(),
                user: "user".to_owned(),
            }),
            ..Default::default()
        };
        let _ = CommonAppState::try_from(&valid_settings).unwrap();
    }
}
