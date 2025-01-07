use clickhouse::Client;
use serde::{Deserialize, Serialize};

/// `MetricsDatabaseSettings` represents the required parameters for connecting
/// to the Clickhouse database instance in which metrics data is being recorded
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetricsDatabaseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub pass: String,
}

impl From<&MetricsDatabaseSettings> for Client {
    fn from(value: &MetricsDatabaseSettings) -> Self {
        Client::default()
            .with_url(value.url.to_owned())
            .with_user(value.user.to_owned())
            .with_password(value.pass.to_owned())
            .with_database(value.database.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use clickhouse::Client;

    use super::MetricsDatabaseSettings;

    #[test]
    fn test_into_client() {
        let test_settings = MetricsDatabaseSettings {
            database: "METRICS".to_owned(),
            pass: "test_pass".to_owned(),
            url: "http://localhost:8123".to_owned(),
            user: "test_user".to_owned(),
        };
        let _: Client = Client::from(&test_settings);
    }
}
