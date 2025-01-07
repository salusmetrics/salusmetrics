use std::time::Duration;

use http::HeaderValue;
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};

use crate::{conf_error::ConfError, settings::CommonSettings};

pub const DEFAULT_TIMEOUT_MILLIS: u64 = 30000;

/// Axum settings for the CorsLayer type that can be common across app types
/// If none specified then no CorsLayer will be created
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CorsSettings {
    pub max_age_secs: Option<u64>,
    pub origins: Vec<String>,
}

impl CorsSettings {
    /// Attempt to created a CORS layer from ENV values. If no CORS ENV values
    /// are provided, then there will not be a CorsLayer created.
    pub fn try_create_cors_layer(&self) -> Result<CorsLayer, ConfError> {
        let CorsSettings {
            max_age_secs,
            origins,
        } = self.to_owned();

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

/// Axum settings for the TimeoutLayer type that can be common across app types
/// If none is specified, then default value will be used
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeoutSettings {
    pub millis: u64,
}

impl Default for TimeoutSettings {
    fn default() -> Self {
        Self {
            millis: DEFAULT_TIMEOUT_MILLIS,
        }
    }
}

impl TimeoutSettings {
    /// create an Axum TimeoutLayer with the specified timeout in milliseconds
    pub fn create_timeout_layer(&self) -> TimeoutLayer {
        TimeoutLayer::new(Duration::from_millis(self.millis))
    }
}

/// Collection of the settings types for Axum Layers that are common across apps
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayerSettings {
    pub cors: Option<CorsSettings>,
    pub timeout: Option<TimeoutSettings>,
}

impl Default for LayerSettings {
    fn default() -> Self {
        LayerSettings {
            cors: None,
            timeout: Some(TimeoutSettings::default()),
        }
    }
}

impl LayerSettings {
    pub fn try_new(app_name: &str) -> Result<Self, ConfError> {
        assert!(!app_name.is_empty());
        Ok(CommonSettings::try_new_from_env(app_name)?
            .layer
            .unwrap_or_default())
    }

    /// Returns an Axum TimeoutLayer that will use ENV values for the timeout in millis.
    /// If no value is provided in the ENV, then the default will be used
    pub fn create_timeout_layer(&self) -> TimeoutLayer {
        self.timeout
            .to_owned()
            .unwrap_or_default()
            .create_timeout_layer()
    }

    /// Attempt to created a CORS layer from ENV values. If no CORS ENV values
    /// are provided, then there will not be a CorsLayer created.
    pub fn try_create_cors_layer(&self) -> Result<Option<CorsLayer>, ConfError> {
        match self.to_owned().cors {
            None => Ok(None),
            Some(cors) => Ok(Some(cors.try_create_cors_layer()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        conf_error::ConfError,
        settings::{
            tests::{cleanup_test_env, create_valid_env, setup_valid_test_env},
            CommonSettings,
        },
    };

    use super::CorsSettings;

    #[test]
    fn test_try_new_layer_settings() {
        // Positive test case
        let app_name = setup_valid_test_env();
        let _ = CommonSettings::try_new_from_env(&app_name)
            .unwrap()
            .layer
            .unwrap();
        cleanup_test_env(&app_name);
    }

    #[test]
    fn test_create_timeout_layer() {
        let _ = create_valid_env().layer.unwrap().create_timeout_layer();
    }

    #[test]
    fn test_try_create_cors_layer() {
        let valid_thin_settings = CorsSettings {
            max_age_secs: None,
            origins: vec![
                "https://test.com".to_owned(),
                "http://localhost:3000".to_owned(),
            ],
        };
        let _ = valid_thin_settings.try_create_cors_layer().unwrap();

        let valid_full_settings = CorsSettings {
            max_age_secs: Some(60),
            origins: vec![
                "https://test.com".to_owned(),
                "http://localhost:3000".to_owned(),
            ],
        };
        let _ = valid_full_settings.try_create_cors_layer().unwrap();

        let invalid_settings = CorsSettings {
            max_age_secs: Some(60),
            origins: Vec::new(),
        };
        assert_eq!(
            invalid_settings.try_create_cors_layer().unwrap_err(),
            ConfError::Cors
        );
    }
}
