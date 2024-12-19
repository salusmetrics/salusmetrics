use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use config::tracing::init_tracing_subscriber;
use http::header::HOST;
use hyper::{HeaderMap, StatusCode};
use ingest::client_event::{ClientEvent, ClientEventType};
use ingest::event_record::{ApiKey, EventRecord, Site};
use std::env;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::instrument;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();

    let timeout_millis: u64 = match env::var("HANDLER_TIMEOUT") {
        Ok(val) => val.parse().expect("Handler timeout is not a number!"),
        Err(_) => 30000,
    };

    let app = Router::new()
        .route("/explore", get(explore))
        .route("/ingest", post(test_ingest))
        .layer(TraceLayer::new_for_http())
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
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[instrument]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[instrument]
async fn test_ingest(headers: HeaderMap, Json(event): Json<ClientEvent>) -> impl IntoResponse {
    let host = headers.get(HOST).unwrap();
    let for_insert = EventRecord::try_from_client_event(
        event,
        ApiKey("123-456".to_string()),
        Site(host.to_str().unwrap().to_string()),
    );

    if for_insert.is_ok() {
        return (StatusCode::OK, Json("for_insert.unwrap()"));
    }
    (StatusCode::BAD_REQUEST, Json("for_insert.unwrap_err()"))
}

#[instrument]
async fn explore() -> impl IntoResponse {
    let valid_ingest_event = ClientEvent {
        event_type: ClientEventType::Visitor,
        id: Uuid::now_v7(),
        attrs: Vec::new(),
    };

    (StatusCode::OK, Json(valid_ingest_event))
}
