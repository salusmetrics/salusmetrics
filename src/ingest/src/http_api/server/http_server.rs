use axum::{Router, routing::post};
use conf::domain::service::configuration_service::ConfigurationService;
use http::Method;
use std::{error::Error, net::SocketAddr};
use tower_http::{cors::Any, trace::TraceLayer};

use crate::{
    http_api::{
        handlers::save_client_events::save_client_events,
        model::ingest_application_state::IngestApplicationState,
    },
    repositories::clickhouse_ingest_repository::ClickhouseIngestRepository,
    services::ingest_service::IngestService,
};

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
        let metrics_client = self.conf_service.try_metrics_db_client()?;

        let compression_layer = self.conf_service.try_compression_layer()?;
        let cors_layer = self
            .conf_service
            .try_cors_layer()?
            .allow_methods([Method::POST])
            .allow_headers(Any);
        let timeout_layer = self.conf_service.try_timeout_layer()?;

        let ingest_repository = ClickhouseIngestRepository::try_new(metrics_client).await?;
        let ingest_service = IngestService::new(ingest_repository);
        let state = IngestApplicationState::new(ingest_service);
        let app = Router::new()
            .route(
                "/multi",
                post(save_client_events::<IngestService<ClickhouseIngestRepository>>),
            )
            .layer(TraceLayer::new_for_http())
            .layer(compression_layer)
            .layer(cors_layer)
            .layer(timeout_layer)
            .with_state(state);

        let listener_socket_addr = self.conf_service.try_listener_socket_addr()?;
        let listener = tokio::net::TcpListener::bind(listener_socket_addr).await?;
        tracing::debug!("listening on {}", listener.local_addr().unwrap());

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(conf::lifecycle::terminate_signal())
        .await
        .unwrap();
        Ok(())
    }
}
