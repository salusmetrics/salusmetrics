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
        let metrics_settings = value.metricsdb().ok_or(ConfError::MetricsDb)?;

        Ok(CommonAppState {
            metrics_db_client: Client::from(&metrics_settings),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{metrics_database::MetricsDatabaseSettings, settings::CommonSettingsBuilder};

    use super::CommonAppState;

    #[test]
    fn test_try_from() {
        //positive test case
        let valid_settings = CommonSettingsBuilder::new()
            .metricsdb(MetricsDatabaseSettings::new(
                "http://localhost:8123",
                "METRICS_DB",
                "user",
                "pass",
            ))
            .build();
        let _ = CommonAppState::try_from(&valid_settings).unwrap();
    }
}
