use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use super::configuration_error::ConfigurationError;

/// `TracingSettings` represents the application-wide settings that will be used
/// to set up a `tracing_subscriber::EnvFilter`. The `directive` is intended to
/// be passed along to `EnvFilter::try_new`.
/// Default settings for tracing subscriber are to filter only ERROR and above
#[derive(Debug, Clone)]
pub struct TracingSettings {
    pub directive: String,
}

impl Default for TracingSettings {
    /// Defaults to filtering all tracing to ERROR and above
    fn default() -> Self {
        TracingSettings {
            directive: "ERROR".to_owned(),
        }
    }
}

impl TryInto<EnvFilter> for &TracingSettings {
    type Error = ConfigurationError;
    fn try_into(self) -> Result<EnvFilter, Self::Error> {
        EnvFilter::try_new(&self.directive).map_err(|_| ConfigurationError::Parse)
    }
}

impl TracingSettings {
    /// Constructor for `TracingSettings`
    pub fn new(directive: impl AsRef<str>) -> Self {
        Self {
            directive: directive.as_ref().to_owned(),
        }
    }

    /// Attempt to initialize the tracing subscriber based on settings
    pub fn try_init_tracing_subscriber(&self) -> Result<(), ConfigurationError> {
        let filter_layer: EnvFilter = self.try_into()?;
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(tracing_subscriber::fmt::layer().compact().with_target(true))
            .init();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::TracingSettings;

    #[test]
    fn test_try_into() {
        let to_test = TracingSettings {
            directive: "error".to_owned(),
        };
        let _: tracing_subscriber::EnvFilter = (&to_test).try_into().unwrap();
    }

    #[test]
    fn test_try_init_tracing_subscriber() {
        let to_test = TracingSettings {
            directive: "error".to_owned(),
        };

        to_test.try_init_tracing_subscriber().unwrap();
    }
}
