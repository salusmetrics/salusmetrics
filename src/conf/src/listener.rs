use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::net::TcpListener;
use tracing::instrument;

use crate::{conf_error::ConfError, settings};

/// Axum handler listener settings
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListenerSettings {
    pub port: u16,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

/// Set up port and address that the service will listen on by examining
/// ENV for the provided app name
#[instrument]
pub async fn try_new_listener(app_name: &str) -> Result<TcpListener, ConfError> {
    let settings = settings::SharedSettings::try_new(app_name)?
        .listener
        .ok_or(ConfError::Listener)?;
    let addr = match (settings.ipv4, settings.ipv6) {
        (Some(_), Some(_)) => {
            tracing::error!("env specified both IPv4 and IPv6 addresses");
            return Err(ConfError::Listener);
        }
        (None, None) => std::net::SocketAddr::from((Ipv4Addr::UNSPECIFIED, settings.port)),
        (Some(v4), None) => std::net::SocketAddr::from((v4, settings.port)),
        (None, Some(v6)) => std::net::SocketAddr::from((v6, settings.port)),
    };

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|_| ConfError::Listener)?;
    Ok(listener)
}

#[cfg(test)]
mod tests {
    use crate::settings::tests::{cleanup_test_env, setup_valid_test_env};

    use super::try_new_listener;

    #[tokio::test]
    async fn test_try_new_listener() {
        // Test the positive case first
        let app_name = setup_valid_test_env();
        try_new_listener(&app_name).await.unwrap();
        cleanup_test_env(&app_name);

        // Test negative case
        let res = try_new_listener("INVALID_APP_NAME").await;
        assert!(res.is_err());
    }
}
