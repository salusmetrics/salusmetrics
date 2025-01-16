use axum::{extract::State, Json};
use tracing::instrument;

use crate::{
    domain::{model::ingest_event::IngestEvent, service::ingest_event_service::IngestEventService},
    http_api::model::{
        client_event_action_summary::ClientEventActionSummary,
        client_event_request::{ClientEventRequest, ClientEventRequestError},
        client_event_request_components::{ClientEventRequestBody, ClientEventRequestHeaders},
        ingest_application_state::IngestApplicationState,
    },
};

/// `save_client_events` expects POST data in JSON format that consists of
/// a list of `ClientEventRequestBody` structs as well as information in the
/// HTTP headers which can be used to determine the `api_key` and the `site`
/// for the incoming request. `site` is determined in a simple fashion by
/// examining the referrer attribute, whereas the api_key uses a custom header
/// as specified in `client_event_request_components::API_KEY_HTTP_HEADER`
#[instrument]
pub async fn save_client_events<I: IngestEventService + std::fmt::Debug>(
    State(state): State<IngestApplicationState<I>>,
    client_request_headers: ClientEventRequestHeaders,
    Json(event_bodies): Json<Vec<ClientEventRequestBody>>,
) -> Result<ClientEventActionSummary, ClientEventRequestError> {
    let requests: Vec<ClientEventRequest> = event_bodies
        .iter()
        .map(|eb| ClientEventRequest {
            body: eb.to_owned(),
            headers: client_request_headers.clone(),
        })
        .collect();
    let mut events: Vec<IngestEvent> = Vec::with_capacity(requests.len());
    for request in requests.iter() {
        events.push(request.try_into()?);
    }

    state
        .ingest_service
        .save(events)
        .await
        .map_err(|e| e.into())
        .map(|s| s.into())
}
