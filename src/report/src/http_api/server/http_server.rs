use axum::{Router, response::Html, routing::get};
use conf::domain::service::configuration_service::ConfigurationService;
use http::Method;
use std::error::Error;
use tower_http::{cors::Any, trace::TraceLayer};

pub struct HttpServer<T>
where
    T: ConfigurationService + Sync + Send,
{
    conf_service: T,
}

impl<T> HttpServer<T>
where
    T: ConfigurationService + Sync + Send,
{
    pub fn new(conf_service: T) -> Self {
        Self { conf_service }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error + 'static>> {
        // let metrics_client = self.conf_service.try_metrics_db_client()?;

        let compression_layer = self.conf_service.try_compression_layer()?;
        let cors_layer = self
            .conf_service
            .try_cors_layer()?
            .allow_methods([Method::POST])
            .allow_headers(Any);
        let timeout_layer = self.conf_service.try_timeout_layer()?;

        let app = Router::new()
            .route("/home", get(home))
            .layer(TraceLayer::new_for_http())
            .layer(compression_layer)
            .layer(cors_layer)
            .layer(timeout_layer);
        // .with_state(state);

        let listener_socket_addr = self.conf_service.try_listener_socket_addr()?;
        let listener = tokio::net::TcpListener::bind(listener_socket_addr).await?;
        tracing::debug!("listening on {}", listener.local_addr().unwrap());

        axum::serve(listener, app)
            .with_graceful_shutdown(conf::lifecycle::terminate_signal())
            .await
            .unwrap();
        Ok(())
    }
}

async fn home() -> Html<&'static str> {
    Html("<div>Home</div>")
}
