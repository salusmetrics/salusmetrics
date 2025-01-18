use std::time::Duration;

use http::HeaderValue;
use tower_http::cors::CorsLayer;
use tracing::instrument;

use super::configuration_error::ConfigurationError;

/// `CorsSettings` represents axum settings for the `CorsLayer` type that is
/// common across app metrics apps. Not all apps require CORS, in which case
/// this setting should not be specified in ENV.
///
/// `origins` is required and must not be empty for the CORS layer to be used
///
/// `max_age_secs` represents the number of seconds allowed between an Options
/// request from the browser and any other method. This is not required and
/// will fall back to the default of zero for the system, which means that
/// every individual request must perform an options handshake.
#[derive(Debug, Clone)]
pub struct CorsSettings {
    pub max_age_secs: Option<u64>,
    pub origins: Vec<String>,
}

impl CorsSettings {
    /// `CorsSettings` constructor
    pub fn new(origins: Vec<String>, max_age_secs: Option<u64>) -> Self {
        assert!(!origins.is_empty());
        Self {
            max_age_secs,
            origins,
        }
    }
}

impl TryFrom<&CorsSettings> for CorsLayer {
    type Error = ConfigurationError;
    #[instrument]
    fn try_from(value: &CorsSettings) -> Result<Self, Self::Error> {
        let CorsSettings {
            max_age_secs,
            origins,
        } = value;

        if origins.is_empty() {
            tracing::error!("Empty list of CORS allowed origins specified");
            return Err(ConfigurationError::Invalid);
        }
        let mut allowed_origins: Vec<HeaderValue> = Vec::with_capacity(origins.len());
        for origin in origins.iter() {
            allowed_origins.push(origin.parse::<HeaderValue>().map_err(|e| {
                tracing::error!("Error parsing CORS origin value: {e}");
                ConfigurationError::Parse
            })?);
        }

        tracing::info!("Accepting CORS requests from: {:?}", &allowed_origins);

        let mut layer = CorsLayer::new().allow_origin(allowed_origins);
        if let Some(secs) = max_age_secs {
            layer = layer.max_age(Duration::from_secs(secs.to_owned()));
        }

        Ok(layer)
    }
}

#[cfg(test)]
mod tests {
    use tower_http::cors::CorsLayer;

    use crate::domain::model::configuration_error::ConfigurationError;

    use super::CorsSettings;

    #[test]
    fn test_cors_settings() {
        // Positive test case
        let valid_no_max_settings = CorsSettings {
            origins: vec!["http://localhost:3000".to_owned()],
            max_age_secs: None,
        };
        let _ = CorsLayer::try_from(&valid_no_max_settings).unwrap();
        let valid_with_max_settings = CorsSettings {
            origins: vec!["http://localhost:3000".to_owned()],
            max_age_secs: Some(10),
        };
        let _ = CorsLayer::try_from(&valid_with_max_settings).unwrap();

        // Negative test case
        let invalid_empty_settings = CorsSettings {
            origins: vec![],
            max_age_secs: None,
        };
        assert_eq!(
            CorsLayer::try_from(&invalid_empty_settings).unwrap_err(),
            ConfigurationError::Invalid
        );
    }
}
