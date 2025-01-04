use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use conf::conf_error::ConfError;
use conf::state::AppState;
use http::Method;
use hyper::{HeaderMap, StatusCode};
use ingest::client_event::{ClientEvent, ClientEventType, EventHeaders};
use ingest::event_record::EventRecord;
use std::error::Error;
use tower_http::cors::Any;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::instrument;
use uuid::Uuid;

/// APP_NAME is used to resolve configuration parameters from ENV
pub const APP_NAME: &str = "SALUS_INGEST";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    conf::tracing::init_tracing_subscriber(APP_NAME)?;

    let state = AppState::try_new(APP_NAME)?;

    let layer_settings = conf::layer::LayerSettings::try_new(APP_NAME)?;
    let cors_layer = layer_settings
        .try_create_cors_layer()?
        .ok_or(ConfError::Cors)?
        .allow_methods([Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/explore", get(explore))
        .route("/ingest", post(test_ingest))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        .layer(layer_settings.create_timeout_layer())
        .with_state(state);

    let listener = conf::listener::try_new_listener(APP_NAME).await?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(conf::lifecycle::terminate_signal())
        .await
        .unwrap();
    Ok(())
}

async fn test_ingest(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<ClientEvent>,
) -> impl IntoResponse {
    let Ok(event_headers) = EventHeaders::try_from(&headers) else {
        return StatusCode::BAD_REQUEST;
    };
    let for_insert =
        EventRecord::try_from_client_event(event, event_headers.api_key, event_headers.site);

    let Ok(record) = for_insert else {
        return StatusCode::BAD_REQUEST;
    };
    tracing::debug!("record: {record:?}");

    let client = app_state.metrics_db_client;
    let Ok(mut insert) = client.insert("EVENT") else {
        return StatusCode::BAD_REQUEST;
    };

    if insert.write(&record).await.is_err() {
        return StatusCode::BAD_REQUEST;
    }
    if insert.end().await.is_err() {
        return StatusCode::BAD_REQUEST;
    }
    StatusCode::OK
}

#[instrument]
async fn explore() -> impl IntoResponse {
    let valid_ingest_event = ClientEvent {
        event_type: ClientEventType::Visitor,
        id: Uuid::now_v7(),
        attrs: Some(Vec::new()),
    };

    (StatusCode::OK, Json(valid_ingest_event))
}
