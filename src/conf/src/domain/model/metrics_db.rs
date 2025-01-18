use clickhouse::Client;

/// `MetricsDatabaseSettings` represents the required parameters for connecting
/// to the Clickhouse database instance in which metrics data is being recorded
#[derive(Debug, Clone)]
pub struct MetricsDatabaseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub pass: String,
}

impl MetricsDatabaseSettings {
    pub fn new(
        url: impl AsRef<str>,
        database: impl AsRef<str>,
        user: impl AsRef<str>,
        pass: impl AsRef<str>,
    ) -> Self {
        Self {
            url: url.as_ref().to_owned(),
            database: database.as_ref().to_owned(),
            user: user.as_ref().to_owned(),
            pass: pass.as_ref().to_owned(),
        }
    }
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
