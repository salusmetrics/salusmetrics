use axum::{extract::State, Json};

use crate::{
    domain::{model::ingest_event::IngestEvent, service::ingest_event_service::IngestEventService},
    http_api::model::{
        client_event_action_summary::ClientEventActionSummary,
        client_event_request::{ClientEventRequest, ClientEventRequestError},
        client_event_request_components::{ClientEventRequestBody, ClientEventRequestHeaders},
        ingest_application_state::IngestApplicationState,
    },
};

pub async fn save_client_events<I: IngestEventService>(
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
