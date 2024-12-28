use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use conf::lifecycle::terminate_signal;
use conf::metrics_database::try_get_metrics_client;
use conf::tracing::init_tracing_subscriber;
use hyper::{HeaderMap, StatusCode};
use ingest::client_event::{ClientEvent, ClientEventType, EventHeaders};
use ingest::event_record::EventRecord;
use std::env;
use std::error::Error;
use std::net::Ipv4Addr;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::instrument;
use uuid::Uuid;

/// APP_NAME is used to resolve configuration parameters from ENV
pub const APP_NAME: &str = "SALUS_INGEST";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    init_tracing_subscriber(APP_NAME)?;

    let timeout_millis: u64 = match env::var("HANDLER_TIMEOUT") {
        Ok(val) => val.parse().expect("Handler timeout is not a number!"),
        Err(_) => 30000,
    };

    let app = Router::new()
        .route("/explore", get(explore))
        .route("/ingest", post(test_ingest))
        .layer(TraceLayer::new_for_http())
        // TODO: narrow down allowed origins and more, plus move to common conf crate
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .max_age(Duration::from_secs(10)),
        )
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        .layer(TimeoutLayer::new(Duration::from_millis(timeout_millis)));

    let port: u16 = match env::var("HANDLER_PORT") {
        Ok(val) => val.parse().expect("Handler port is not a number!"),
        Err(_) => 3000,
    };

    let addr = std::net::SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(terminate_signal())
        .await
        .unwrap();
    Ok(())
}

#[instrument]
async fn test_ingest(headers: HeaderMap, Json(event): Json<ClientEvent>) -> impl IntoResponse {
    let Ok(event_headers) = EventHeaders::try_from(&headers) else {
        return StatusCode::BAD_REQUEST;
    };
    let for_insert =
        EventRecord::try_from_client_event(event, event_headers.api_key, event_headers.site);

    let Ok(record) = for_insert else {
        return StatusCode::BAD_REQUEST;
    };
    tracing::debug!("record: {record:?}");

    let Ok(client) = try_get_metrics_client(APP_NAME) else {
        return StatusCode::BAD_REQUEST;
    };
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
