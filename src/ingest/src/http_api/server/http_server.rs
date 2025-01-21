use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use conf::domain::service::configuration_service::ConfigurationService;
use http::{Method, StatusCode};
use std::{collections::HashMap, error::Error};
use tower_http::{cors::Any, trace::TraceLayer};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    http_api::{
        handlers::save_client_events::save_client_events,
        model::{
            client_event_request::ClientEventRequestType,
            client_event_request_components::ClientEventRequestBody,
            ingest_application_state::IngestApplicationState,
        },
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

        let ingest_repository = ClickhouseIngestRepository::new(metrics_client);
        let ingest_service = IngestService::new(ingest_repository);
        let state = IngestApplicationState::new(ingest_service);
        let app = Router::new()
            .route(
                "/multi",
                post(save_client_events::<IngestService<ClickhouseIngestRepository>>),
            )
            .route("/explore", get(explore))
            .layer(TraceLayer::new_for_http())
            .layer(compression_layer)
            .layer(cors_layer)
            .layer(timeout_layer)
            .with_state(state);

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

// TODO: remove this test function

#[instrument]
async fn explore() -> impl IntoResponse {
    let visitor_uuid = Uuid::now_v7();
    let session_uuid = Uuid::now_v7();
    let section_uuid = Uuid::now_v7();
    let mut session_attrs: HashMap<String, String> = HashMap::new();
    session_attrs.insert("parent".to_owned(), visitor_uuid.to_string());
    let mut section_attrs: HashMap<String, String> = HashMap::new();
    section_attrs.insert("parent".to_owned(), session_uuid.to_string());
    let visitor_event = ClientEventRequestBody::new(
        ClientEventRequestType::Visitor,
        visitor_uuid.to_owned(),
        None,
    );
    let session_event = ClientEventRequestBody::new(
        ClientEventRequestType::Session,
        session_uuid.to_owned(),
        Some(session_attrs),
    );
    let section_event = ClientEventRequestBody::new(
        ClientEventRequestType::Section,
        section_uuid.to_owned(),
        Some(section_attrs),
    );

    let all_events: Vec<ClientEventRequestBody> = vec![visitor_event, session_event, section_event];

    (StatusCode::OK, Json(all_events))
}
