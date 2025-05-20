use axum_client_ip::ClientIpSource;

/// `IpSourceSettings` allows specification of how the client IP should be
/// determined by the axum_client_ip crate. This IP is used to determine a
/// geolocation for the remote http client. The value for this must map to
/// `axum_client_ip::ClientIpSource`. The setting is optional and will default
/// to `axum_client_ip::ClientIpSource::ConnectInfo`, which looks at the IP
/// directly included in the request, rather than headers attached via a proxy
/// like cloudflare or other service
#[derive(Debug, Clone)]
pub struct IpSourceSettings {
    pub variant: ClientIpSource,
}

impl Default for IpSourceSettings {
    fn default() -> Self {
        Self {
            variant: ClientIpSource::ConnectInfo,
        }
    }
}

impl From<&IpSourceSettings> for ClientIpSource {
    fn from(value: &IpSourceSettings) -> Self {
        value.variant.clone()
    }
}

#[cfg(test)]
mod tests {
    use axum_client_ip::ClientIpSource;

    use super::*;

    #[test]
    fn test_axum_extension() {
        // Test default value works using just IP from connection
        let connect_info = IpSourceSettings::default();
        let _ = ClientIpSource::from(&connect_info).into_extension();

        // Test alternate sources explicitly
        // Cloudflare
        let cf_connecting = IpSourceSettings {
            variant: ClientIpSource::CfConnectingIp,
        };
        let _ = ClientIpSource::from(&cf_connecting).into_extension();
        // RightmostForwarded
        let rightmost = IpSourceSettings {
            variant: ClientIpSource::RightmostForwarded,
        };
        let _ = ClientIpSource::from(&rightmost).into_extension();
    }
}
