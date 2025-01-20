use clickhouse::Client;

/// `MetricsDatabaseSettings` represents the required parameters for connecting
/// to the Clickhouse database instance in which metrics data is being recorded
#[derive(Clone)]
pub struct MetricsDatabaseSettings {
    url: String,
    database: String,
    user: String,
    pass: String,
}

impl std::fmt::Debug for MetricsDatabaseSettings {
    /// `fmt` implemented for `Debug` to prevent leakage of username and
    /// password for the database into logs or traces
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("url", &self.url)
            .field("database", &self.database)
            .finish()
    }
}

impl MetricsDatabaseSettings {
    /// `MetricsDatabaseSettings` simple constructor
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
