use std::time::Duration;

use tower_http::timeout::TimeoutLayer;

pub const DEFAULT_TIMEOUT_MILLIS: u64 = 30000;

/// `TimeoutSettings` allows the customization of a given app's TimeoutLayer
/// which determines how long the server will wait before responding with a
/// timeout. If none is specified, then default value will be used.
/// This will default to a setting of 30 seconds.
#[derive(Debug, Clone)]
pub struct TimeoutSettings {
    pub millis: u64,
}

impl TimeoutSettings {
    /// `TimeoutSettings` constructor
    pub fn new(millis: u64) -> Self {
        Self { millis }
    }
}

impl Default for TimeoutSettings {
    /// Default to a timeout of 30 seconds
    fn default() -> Self {
        Self {
            millis: DEFAULT_TIMEOUT_MILLIS,
        }
    }
}

impl From<&TimeoutSettings> for TimeoutLayer {
    fn from(value: &TimeoutSettings) -> Self {
        TimeoutLayer::new(Duration::from_millis(value.millis))
    }
}

#[cfg(test)]
mod tests {
    use tower_http::timeout::TimeoutLayer;

    use super::*;

    #[test]
    fn test_timeout_settings() {
        // Positive test case
        let test_settings = TimeoutSettings { millis: 60000 };
        let _ = TimeoutLayer::from(&test_settings);
    }
}
