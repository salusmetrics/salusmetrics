use std::time::Duration;

use http::HeaderValue;
use serde::{Deserialize, Serialize};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer};
use tracing::instrument;

use crate::conf_error::ConfError;

pub const DEFAULT_TIMEOUT_MILLIS: u64 = 30000;

/// `CompressionSettings` allows the setup of `tower-http` `CompressionLayer`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompressionSettings {
    gzip: bool,
}

impl From<&CompressionSettings> for CompressionLayer {
    fn from(value: &CompressionSettings) -> Self {
        if value.gzip {
            CompressionLayer::new().gzip(true).deflate(true)
        } else {
            CompressionLayer::new()
        }
    }
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self { gzip: true }
    }
}

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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CorsSettings {
    max_age_secs: Option<u64>,
    origins: Vec<String>,
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
    type Error = ConfError;
    #[instrument]
    fn try_from(value: &CorsSettings) -> Result<Self, Self::Error> {
        let CorsSettings {
            max_age_secs,
            origins,
        } = value.to_owned();

        if origins.is_empty() {
            tracing::error!("Empty list of CORS allowed origins specified");
            return Err(ConfError::Cors);
        }
        let mut allowed_origins: Vec<HeaderValue> = Vec::with_capacity(origins.len());
        for origin in origins.iter() {
            allowed_origins.push(origin.parse::<HeaderValue>().map_err(|e| {
                tracing::error!("Error parsing CORS origin value: {e}");
                ConfError::Cors
            })?);
        }

        tracing::info!("Accepting CORS requests from: {:?}", &allowed_origins);

        let mut layer = CorsLayer::new().allow_origin(allowed_origins);
        if let Some(secs) = max_age_secs {
            layer = layer.max_age(Duration::from_secs(secs));
        }

        Ok(layer)
    }
}

/// `TimeoutSettings` allows the customization of a given app's TimeoutLayer
/// which determines how long the server will wait before responding with a
/// timeout. If none is specified, then default value will be used
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeoutSettings {
    millis: u64,
}

impl TimeoutSettings {
    /// `TimeoutSettings` constructor
    pub fn new(millis: u64) -> Self {
        Self { millis }
    }
}

impl Default for TimeoutSettings {
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

/// `LayerSettings` wraps the `CorsSettings` and `TimeoutSettings` into a
/// common struct which can be used to handle both in a clean manner
/// which will optionally set up a CORS layer if any is specified.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayerSettings {
    compression: Option<CompressionSettings>,
    cors: Option<CorsSettings>,
    timeout: Option<TimeoutSettings>,
}

impl Default for LayerSettings {
    fn default() -> Self {
        LayerSettings {
            compression: Some(CompressionSettings::default()),
            cors: None,
            timeout: Some(TimeoutSettings::default()),
        }
    }
}

impl LayerSettings {
    /// `LayerSettings` constructor
    pub fn new(
        compression_settings: Option<CompressionSettings>,
        cors_settings: Option<CorsSettings>,
        timeout_settings: Option<TimeoutSettings>,
    ) -> Self {
        Self {
            compression: compression_settings,
            cors: cors_settings,
            timeout: timeout_settings,
        }
    }

    /// Attempt to create a `tower-http` `CompressionLayer` from this
    /// `LayerSettings`. Will return None if `compression_settings` is None
    pub fn try_create_compression_layer(&self) -> Option<CompressionLayer> {
        Some(CompressionLayer::from(&self.compression.to_owned()?))
    }

    /// Attempt to create an axum `CorsLayer` if one is spefified, otherwise
    /// returns None. In order to be a valid setup, the list of origins must
    /// not be empty. If no max_age_secs is specified, then the axum default
    /// will be used, which causes all requests to start with a handshake
    pub fn try_create_cors_layer(&self) -> Result<Option<CorsLayer>, ConfError> {
        match self.cors.to_owned() {
            None => Ok(None),
            Some(cors) => Ok(Some(CorsLayer::try_from(&cors)?)),
        }
    }

    /// Return an axum `TimeoutLayer` that reflects the vakues of the
    /// TimeoutSettings field or else falls back to the default value.
    pub fn create_timeout_layer(&self) -> TimeoutLayer {
        TimeoutLayer::from(&self.timeout.to_owned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer};

    use crate::conf_error::ConfError;

    use super::{CompressionSettings, CorsSettings, LayerSettings, TimeoutSettings};

    #[test]
    fn test_compression_settings() {
        // Positive test case
        let test_settings = CompressionSettings::default();
        let _ = CompressionLayer::from(&test_settings);
    }

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
        let invalid_settings = CorsSettings {
            origins: vec![],
            max_age_secs: None,
        };
        assert_eq!(
            CorsLayer::try_from(&invalid_settings).unwrap_err(),
            ConfError::Cors
        );
    }

    #[test]
    fn test_timeout_settings() {
        // Positive test case
        let test_settings = TimeoutSettings { millis: 60000 };
        let _ = TimeoutLayer::from(&test_settings);
    }

    #[test]
    fn test_try_create_compression_layer() {
        // Default case should create a CompressionLayer with gzip enabled
        let default_settings = LayerSettings::default();
        if default_settings.try_create_compression_layer().is_none() {
            panic!("expected CompressionLayer to be created");
        }

        let none_settings = LayerSettings {
            compression: None,
            ..Default::default()
        };
        if none_settings.try_create_compression_layer().is_some() {
            panic!("expected None for CompressionLayer");
        }
    }

    #[test]
    fn test_try_create_cors_layer() {
        // Default case should succeed, but with no CorsLayer returned
        let default_settings = LayerSettings::default();
        if default_settings.try_create_cors_layer().unwrap().is_some() {
            panic!("expected None");
        };

        // Create a valid CorsLayer
        let valid_settings = LayerSettings {
            cors: Some(CorsSettings {
                max_age_secs: None,
                origins: vec!["http://127.0.0.1:9000".to_owned()],
            }),
            ..Default::default()
        };
        if valid_settings.try_create_cors_layer().unwrap().is_none() {
            panic!("expected valid CorsLayer to be created");
        }

        // Invalid CorsSettings
        let invalid_settings = LayerSettings {
            cors: Some(CorsSettings {
                max_age_secs: None,
                origins: vec![],
            }),
            ..Default::default()
        };
        assert_eq!(
            invalid_settings.try_create_cors_layer().unwrap_err(),
            ConfError::Cors
        );
    }

    #[test]
    fn test_create_timeout_layer() {
        let default_settings = LayerSettings::default();
        let _ = default_settings.create_timeout_layer();

        let specified_timeout = LayerSettings {
            timeout: Some(TimeoutSettings { millis: 500 }),
            ..Default::default()
        };
        specified_timeout.create_timeout_layer();
    }
}
