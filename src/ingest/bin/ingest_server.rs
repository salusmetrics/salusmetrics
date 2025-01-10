use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use clickhouse::insert::Insert;
use conf::conf_error::ConfError;
use conf::settings::CommonSettings;
use conf::state::CommonAppState;
use http::Method;
use hyper::{HeaderMap, StatusCode};
use ingest::client_event::{ClientEvent, ClientEventBody, ClientEventType, EventHeaders};
use ingest::event_record::EventRecord;
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

    let state = CommonAppState::try_from(&env_settings)?;

    let mut app = Router::new()
        .route("/explore", get(explore))
        .route("/ingest", post(test_ingest))
        .route("/multi", post(test_multi_ingest))
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

async fn test_multi_ingest(
    State(app_state): State<CommonAppState>,
    headers: HeaderMap,
    Json(event_bodies): Json<Vec<ClientEventBody>>,
) -> impl IntoResponse {
    let Ok(event_headers) = EventHeaders::try_from(&headers) else {
        return StatusCode::BAD_REQUEST;
    };

    let events: Vec<ClientEvent> = event_bodies
        .iter()
        .map(|eb| ClientEvent::new(&event_headers, eb))
        .collect();
    let mut event_records: Vec<EventRecord> = Vec::with_capacity(events.len());
    for ev in events.iter() {
        let Ok(er) = EventRecord::try_from(ev) else {
            return StatusCode::BAD_REQUEST;
        };
        event_records.push(er);
    }
    tracing::debug!("records: {event_records:?}");

    let client = app_state.metrics_db_client;
    let Ok(mut insert) = client.insert::<EventRecord>("EVENT") else {
        return StatusCode::BAD_REQUEST;
    };

    for er in event_records.iter() {
        if insert.write(er).await.is_err() {
            return StatusCode::BAD_REQUEST;
        }
    }
    if insert.end().await.is_err() {
        return StatusCode::BAD_REQUEST;
    }
    StatusCode::OK
}

async fn test_ingest(
    State(app_state): State<CommonAppState>,
    headers: HeaderMap,
    Json(event): Json<ClientEventBody>,
) -> impl IntoResponse {
    let Ok(event_headers) = EventHeaders::try_from(&headers) else {
        return StatusCode::BAD_REQUEST;
    };
    let event = ClientEvent::new(&event_headers, &event);
    let for_insert = EventRecord::try_from(&event);

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
    let visitor_uuid = Uuid::now_v7();
    let session_uuid = Uuid::now_v7();
    let section_uuid = Uuid::now_v7();
    let session_attrs: Vec<(String, String)> =
        vec![("parent".to_owned(), visitor_uuid.to_string())];
    let section_attrs: Vec<(String, String)> =
        vec![("parent".to_owned(), session_uuid.to_string())];
    let visitor_event =
        ClientEventBody::new(ClientEventType::Visitor, visitor_uuid.to_owned(), None);
    let session_event = ClientEventBody::new(
        ClientEventType::Session,
        session_uuid.to_owned(),
        Some(session_attrs),
    );
    let section_event = ClientEventBody::new(
        ClientEventType::Section,
        section_uuid.to_owned(),
        Some(section_attrs),
    );

    let all_events: Vec<ClientEventBody> = vec![visitor_event, session_event, section_event];

    (StatusCode::OK, Json(all_events))
}
