use std::net::SocketAddr;

use clickhouse::Client;
use thiserror::Error;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer};

/// `ConfigurationServiceError` represents the domain errors that can arise
/// when calling a given `ConfigurationService`
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ConfigurationServiceError {
    /// `Invalid` arises when the configuration specified for the
    /// request type is not valid and cannot create the desired result type
    #[error("Configuration for this type was not valid")]
    Invalid,
    /// `Missing` arises when the configuration repository does
    /// not contain any settings for this particular config type
    #[error("Could not find configuration information for the requested type")]
    Missing,
    /// `Repository` arises when the undrlying repo experiences an unspecified
    /// error
    #[error("Error utilizing repository to fetch configuration")]
    Repository,
}

/// `ConfigurationService` trait provides the interface for fetching settings
/// for an application. This includes a wide range of configuration options
/// from tracing settings to database clients and HTTP listener setup.
pub trait ConfigurationService: 'static + Send + Sync {
    /// `try_compression_layer` attempts to configure and return
    /// `tower_http::compression::CompressionLayer`
    fn try_compression_layer(&self) -> Result<CompressionLayer, ConfigurationServiceError>;

    /// `try_cors_layer` attempts to create and return
    /// `tower_http::cors::CorsLayer`
    fn try_cors_layer(&self) -> Result<CorsLayer, ConfigurationServiceError>;

    /// `try_timeout_layer` attempts to create and return a
    /// `tower_http::timeout::TimeoutLayer`
    fn try_timeout_layer(&self) -> Result<TimeoutLayer, ConfigurationServiceError>;

    /// `try_metrics_db_client` attempts to create and return a
    /// `clickhouse::Client` client to access the Clickhouse database for this
    /// application
    fn try_metrics_db_client(&self) -> Result<Client, ConfigurationServiceError>;

    /// `try_listener_socket_addr` attempts to set up and return a
    /// `std::net::SocketAddr` based on the settings for this application. The
    /// expectation is that this value would be passed along into
    /// `tokio::net::TcpListener::bind()` to bind this application to the
    /// specified IP and port
    fn try_listener_socket_addr(&self) -> Result<SocketAddr, ConfigurationServiceError>;

    /// `try_tracing_subscriber_setup` attempts to configure the Tokio
    /// `tracing_subscriber` based on settings for this application.
    fn try_tracing_subscriber_setup(&self) -> Result<(), ConfigurationServiceError>;
}
