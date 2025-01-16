use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use clickhouse::Client;
use conf::conf_error::ConfError;
use conf::settings::CommonSettings;
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
    let env_settings = CommonSettings::try_new_from_env(APP_NAME)?;
    env_settings.tracing().try_init_tracing_subscriber()?;

    let layer_settings = env_settings.layer().ok_or(ConfError::Layer)?;
    let cors_layer = layer_settings
        .try_create_cors_layer()?
        .ok_or(ConfError::Cors)?
        .allow_methods([Method::POST])
        .allow_headers(Any);

    let metrics_client: Client = (&env_settings.metricsdb().ok_or(ConfError::MetricsDb)?).into();
    let ingest_repository = ClickhouseIngestRepository::new(metrics_client);
    let ingest_service = IngestService::new(ingest_repository);
    let state = IngestApplicationState::new(ingest_service);
    let mut app = Router::new()
        .route("/explore", get(explore))
        // .route("/ingest", post(test_ingest))
        .route(
            "/multi",
            post(save_client_events::<IngestService<ClickhouseIngestRepository>>),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(layer_settings.create_timeout_layer())
        .with_state(state);

    if let Some(compression_layer) = layer_settings.try_create_compression_layer() {
        app = app.layer(compression_layer);
    }

    let listener = env_settings
        .listener()
        .ok_or(ConfError::Listener)?
        .try_new_listener()
        .await?;
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
