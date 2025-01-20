use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use conf::domain::service::configuration_service::ConfigurationService;
use conf::env_conf::env_conf;
use http::Method;
use hyper::StatusCode;
use ingest::http_api::handlers::save_client_events::save_client_events;
use ingest::http_api::model::client_event_request::ClientEventRequestType;
use ingest::http_api::model::client_event_request_components::ClientEventRequestBody;
use ingest::http_api::model::ingest_application_state::IngestApplicationState;
use ingest::repositories::clickhouse_ingest_repository::ClickhouseIngestRepository;
use ingest::services::ingest_service::IngestService;
use std::collections::HashMap;
use std::error::Error;
use tower_http::cors::Any;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use uuid::Uuid;

/// APP_NAME is used to resolve configuration parameters from ENV
pub const APP_NAME: &str = "SALUS_INGEST";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    let conf_service = env_conf(APP_NAME)?;

    conf_service.try_tracing_subscriber_setup()?;

    let metrics_client = conf_service.try_metrics_db_client()?;

    let compression_layer = conf_service.try_compression_layer()?;
    let cors_layer = conf_service
        .try_cors_layer()?
        .allow_methods([Method::POST])
        .allow_headers(Any);
    let timeout_layer = conf_service.try_timeout_layer()?;

    let ingest_repository = ClickhouseIngestRepository::new(metrics_client);
    let ingest_service = IngestService::new(ingest_repository);
    let state = IngestApplicationState::new(ingest_service);
    let app = Router::new()
        .route("/explore", get(explore))
        // .route("/ingest", post(test_ingest))
        .route(
            "/multi",
            post(save_client_events::<IngestService<ClickhouseIngestRepository>>),
        )
        .layer(TraceLayer::new_for_http())
        .layer(compression_layer)
        .layer(cors_layer)
        .layer(timeout_layer)
        .with_state(state);

    let listener_socket_addr = conf_service.try_listener_socket_addr()?;
    let listener = tokio::net::TcpListener::bind(listener_socket_addr).await?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(conf::lifecycle::terminate_signal())
        .await
        .unwrap();
    Ok(())
}

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
