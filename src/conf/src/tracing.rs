use crate::{conf_error::ConfError, settings::SharedSettings};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Tracing settings - ENV variables use the `TRACING` sub-prefix
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TracingSettings {
    pub directive: String,
}

/// Common function to set up tracing subscriber across services.
pub fn init_tracing_subscriber(app_name: &str) -> Result<(), ConfError> {
    assert!(!app_name.is_empty());

    let settings = SharedSettings::try_new(app_name)?;
    let filter_layer = tracing_subscriber::EnvFilter::try_new(settings.tracing.directive)
        .map_err(|_| ConfError::Tracing)?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer().compact().with_target(true))
        .init();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        conf_error::ConfError,
        settings::tests::{cleanup_test_env, setup_valid_test_env, APP_NAME},
    };

    use super::init_tracing_subscriber;

    #[test]
    fn test_init_tracing_subscriber() {
        // test positive case
        setup_valid_test_env();
        init_tracing_subscriber(APP_NAME).unwrap();
        cleanup_test_env();

        // test negative case
        assert_eq!(
            init_tracing_subscriber(APP_NAME).unwrap_err(),
            ConfError::Env
        );
    }
}
