use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tracing::instrument;

use crate::conf_error::ConfError;

/// `ListenerSettings` are used to determine the HTTP listener characteristics
/// of a given metrics application. These include IPv4 or IPv6 address
/// (exclusive) should be attached to as well as the port.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListenerSettings {
    pub port: u16,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

impl TryFrom<&ListenerSettings> for SocketAddr {
    type Error = ConfError;

    #[instrument]
    fn try_from(value: &ListenerSettings) -> Result<Self, Self::Error> {
        match (value.ipv4, value.ipv6) {
            (Some(_), Some(_)) => {
                tracing::error!("env specified both IPv4 and IPv6 addresses");
                Err(ConfError::Listener)
            }
            (None, None) => Ok(SocketAddr::from((Ipv4Addr::UNSPECIFIED, value.port))),
            (Some(v4), None) => Ok(SocketAddr::from((v4, value.port))),
            (None, Some(v6)) => Ok(SocketAddr::from((v6, value.port))),
        }
    }
}

impl ListenerSettings {
    /// Attempt to bind a listener to the specified IP and port for this
    /// `ListenerSettings` struct
    #[instrument]
    pub async fn try_new_listener(&self) -> Result<TcpListener, ConfError> {
        tokio::net::TcpListener::bind(SocketAddr::try_from(self)?)
            .await
            .map_err(|_| ConfError::Listener)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

    use crate::conf_error::ConfError;

    use super::ListenerSettings;

    #[test]
    fn test_try_from() {
        // Valid cases
        let valid_just_port_settings = ListenerSettings {
            ipv4: None,
            ipv6: None,
            port: 8080,
        };
        let _ = SocketAddr::try_from(&valid_just_port_settings).unwrap();

        let valid_v4_settings = ListenerSettings {
            ipv4: Some(Ipv4Addr::LOCALHOST),
            ipv6: None,
            port: 8080,
        };
        let _ = SocketAddr::try_from(&valid_v4_settings).unwrap();

        let valid_v6_settings = ListenerSettings {
            ipv4: None,
            ipv6: Some(Ipv6Addr::LOCALHOST),
            port: 8080,
        };
        let _ = SocketAddr::try_from(&valid_v6_settings).unwrap();

        // Failure case expected
        let invalid_settings = ListenerSettings {
            ipv4: Some(Ipv4Addr::LOCALHOST),
            ipv6: Some(Ipv6Addr::LOCALHOST),
            port: 8080,
        };
        assert_eq!(
            SocketAddr::try_from(&invalid_settings).unwrap_err(),
            ConfError::Listener
        );
    }

    #[tokio::test]
    async fn test_try_new_listener() {
        // Test negative case
        let invalid_settings = ListenerSettings {
            ipv4: Some(Ipv4Addr::LOCALHOST),
            ipv6: Some(Ipv6Addr::LOCALHOST),
            port: 8080,
        };
        assert_eq!(
            invalid_settings.try_new_listener().await.unwrap_err(),
            ConfError::Listener
        );

        // Positive test cases
        let valid_just_port_settings = ListenerSettings {
            ipv4: None,
            ipv6: None,
            port: 8080,
        };
        valid_just_port_settings.try_new_listener().await.unwrap();

        let valid_v4_settings = ListenerSettings {
            ipv4: Some(Ipv4Addr::LOCALHOST),
            ipv6: None,
            port: 3000,
        };
        valid_v4_settings.try_new_listener().await.unwrap();

        let valid_v6_settings = ListenerSettings {
            ipv4: None,
            ipv6: Some(Ipv6Addr::LOCALHOST),
            port: 30000,
        };
        valid_v6_settings.try_new_listener().await.unwrap();
    }
}
