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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::{
        domain::{
            model::ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary},
            repository::ingest_event_repository::test::MockIngestEventRepository,
        },
        http_api::model::client_event_request::ClientEventRequestType,
        services::ingest_service::IngestService,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn test_save_client_events() {
        let uuid_now = Uuid::now_v7();

        // Valid success case with non-empty request
        let mock_success_repo = MockIngestEventRepository {
            save_result: Ok(IngestActionSummary::Save(IngestEventSaveSummary {
                event_count: 1,
            })),
        };
        let test_success_service = IngestService {
            ingest_event_repository: mock_success_repo,
        };
        let test_success_state = IngestApplicationState::new(test_success_service);
        let valid_request_bodies: Vec<ClientEventRequestBody> = vec![ClientEventRequestBody {
            attrs: None,
            event_type: ClientEventRequestType::Visitor,
            id: uuid_now,
        }];
        let save_client_events_success = save_client_events(
            State(test_success_state.clone()),
            ClientEventRequestHeaders {
                api_key: "abc-123".to_owned(),
                site: "test.com".to_owned(),
            },
            Json(valid_request_bodies),
        )
        .await;
        let Ok(ClientEventActionSummary::Save(save_summary)) = save_client_events_success else {
            panic!("Expected successful save from HTTP mock");
        };
        assert_eq!(
            save_summary.event_count, 1,
            "Expected to have 1 save event count"
        );
        // Functional repo, but bad request
        let invalid_request_bodies: Vec<ClientEventRequestBody> = Vec::new();
        let save_client_events_invalid = save_client_events(
            State(test_success_state.clone()),
            ClientEventRequestHeaders {
                api_key: "abc-123".to_owned(),
                site: "test.com".to_owned(),
            },
            Json(invalid_request_bodies),
        )
        .await;
        let Err(ClientEventRequestError::IngestService(_)) = save_client_events_invalid else {
            panic!("Expected error from HTTP mock");
        };
    }
}
