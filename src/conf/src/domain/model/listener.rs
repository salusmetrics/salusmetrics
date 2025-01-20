use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tracing::instrument;

use super::configuration_error::ConfigurationError;

/// `ListenerSettings` are used to determine the HTTP listener characteristics
/// of a given metrics application. These include the IPv4 or IPv6 address
/// (exclusive) that should be attached to as well as the port. Port value
/// is always required. If no IPv4 or IPv6 address has been specified then
/// the configuration will fall back to a default of `Ipv4Addr::UNSPECIFIED`
/// which corresponds to `0.0.0.0`
#[derive(Debug, Clone)]
pub struct ListenerSettings {
    pub port: u16,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

impl TryFrom<&ListenerSettings> for SocketAddr {
    type Error = ConfigurationError;

    #[instrument]
    fn try_from(value: &ListenerSettings) -> Result<Self, Self::Error> {
        match (value.ipv4, value.ipv6) {
            (Some(_), Some(_)) => {
                tracing::error!("env specified both IPv4 and IPv6 addresses");
                Err(ConfigurationError::Invalid)
            }
            (None, None) => Ok(SocketAddr::from((Ipv4Addr::UNSPECIFIED, value.port))),
            (Some(v4), None) => Ok(SocketAddr::from((v4, value.port))),
            (None, Some(v6)) => Ok(SocketAddr::from((v6, value.port))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

    use crate::domain::model::configuration_error::ConfigurationError;

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
            ConfigurationError::Invalid
        );
    }
}
