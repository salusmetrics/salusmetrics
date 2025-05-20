use std::net::{Ipv4Addr, Ipv6Addr};

use axum_client_ip::ClientIpSource;
use serde::{Deserialize, Serialize};

use crate::domain::model::{
    compression::CompressionSettings, cors::CorsSettings, ip_source::IpSourceSettings,
    listener::ListenerSettings, metrics_db::MetricsDatabaseSettings, timeout::TimeoutSettings,
    tracing::TracingSettings,
};

/// `EnvCompressionSettings` allows the setup of `tower-http` `CompressionLayer`
/// `gzip` and `deflate` are booleans that control those attributes respectively
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvCompressionSettings {
    gzip: Option<bool>,
    deflate: Option<bool>,
}

impl From<&EnvCompressionSettings> for CompressionSettings {
    fn from(value: &EnvCompressionSettings) -> Self {
        Self {
            gzip: value.gzip,
            deflate: value.deflate,
        }
    }
}

/// `EnvCorsSettings` represents axum settings for the `CorsLayer` type that is
/// common across app metrics apps. Not all apps require CORS, in which case
/// this setting should not be specified in ENV.
///
/// `origins` is required and must not be empty for the CORS layer to be used
///
/// `max_age_secs` represents the number of seconds allowed between an Options
/// request from the browser and any other method. This is not required and
/// will fall back to the default of zero for the system, which means that
/// every individual request must perform an options handshake.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvCorsSettings {
    max_age_secs: Option<u64>,
    origins: Vec<String>,
}

impl From<&EnvCorsSettings> for CorsSettings {
    fn from(value: &EnvCorsSettings) -> Self {
        Self {
            max_age_secs: value.max_age_secs,
            origins: value.origins.clone(),
        }
    }
}

/// `EnvListenerSettings` are used to determine the HTTP listener characteristics
/// of a given metrics application. These include IPv4 or IPv6 address
/// (exclusive) should be attached to as well as the port.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvListenerSettings {
    port: u16,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}

impl From<&EnvListenerSettings> for ListenerSettings {
    fn from(value: &EnvListenerSettings) -> Self {
        Self {
            port: value.port,
            ipv4: value.ipv4,
            ipv6: value.ipv6,
        }
    }
}

/// `EnvMetricsDatabaseSettings` represents the required parameters for connecting
/// to the Clickhouse database instance in which metrics data is being recorded.
/// All fields must be specified in order to derive a valid database connection
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvMetricsDatabaseSettings {
    url: String,
    database: String,
    user: String,
    pass: String,
}

impl From<&EnvMetricsDatabaseSettings> for MetricsDatabaseSettings {
    fn from(value: &EnvMetricsDatabaseSettings) -> Self {
        Self::new(&value.url, &value.database, &value.user, &value.pass)
    }
}

/// `IpSettings` allows the user to set up how the sytems will determine
/// the remote HTTP client IP address. This can be from the network layer,
/// or via headers that are added by proxy tiers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnvIpSettings {
    source: ClientIpSource,
}

impl From<&EnvIpSettings> for IpSourceSettings {
    fn from(value: &EnvIpSettings) -> Self {
        Self {
            variant: value.source.to_owned(),
        }
    }
}

/// `TimeoutSettings` allows the customization of a given app's TimeoutLayer
/// which determines how long the server will wait before responding with a
/// timeout. If none is specified, then default value will be used. The value
/// accepted is in milliseconds
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvTimeoutSettings {
    millis: u64,
}

impl From<&EnvTimeoutSettings> for TimeoutSettings {
    fn from(value: &EnvTimeoutSettings) -> Self {
        Self {
            millis: value.millis,
        }
    }
}

/// `EnvTracingSettings` represents the application-wide settings that will be used
/// to set up a `tracing_subscriber::EnvFilter`. The `directive` is intended to
/// be passed along to `EnvFilter::try_new`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvTracingSettings {
    directive: String,
}

impl From<&EnvTracingSettings> for TracingSettings {
    fn from(value: &EnvTracingSettings) -> Self {
        Self {
            directive: value.directive.to_owned(),
        }
    }
}
