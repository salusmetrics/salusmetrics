use conf::domain::service::configuration_service::ConfigurationService;
use conf::env_conf::env_conf;
use report::http_api::server::http_server::HttpServer;
use std::error::Error;

/// APP_NAME is used to resolve configuration parameters from ENV
pub const APP_NAME: &str = "SALUS_REPORT";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    let conf_service = env_conf(APP_NAME)?;
    conf_service.try_tracing_subscriber_setup()?;

    let server = HttpServer::new(conf_service);
    server.start().await
}
