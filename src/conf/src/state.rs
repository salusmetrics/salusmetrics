use clickhouse::Client;

use crate::conf_error::ConfError;
use crate::metrics_database::try_get_metrics_client;

/// State struct that provides information like the Clickhouse DB Client
#[derive(Clone)]
pub struct AppState {
    pub metrics_db_client: Client,
}

impl AppState {
    pub fn try_new(app_name: &str) -> Result<Self, ConfError> {
        assert!(!app_name.is_empty());

        let metrics_db_client = try_get_metrics_client(app_name)?;
        Ok(AppState { metrics_db_client })
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::tests::{cleanup_test_env, setup_valid_test_env};

    use super::AppState;

    #[test]
    fn test_try_new() {
        //positive test case
        let app_name = setup_valid_test_env();
        AppState::try_new(&app_name).unwrap();
        cleanup_test_env(&app_name);

        // negative test case
        assert!(AppState::try_new("INVALID_APP_NAME").is_err());
    }
}
