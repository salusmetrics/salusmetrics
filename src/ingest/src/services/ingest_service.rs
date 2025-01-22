use tracing::instrument;

use crate::domain::{
    model::ingest_action_summary::IngestActionSummary,
    repository::ingest_event_repository::{IngestEventRepository, IngestRepositoryError},
    service::ingest_event_service::{IngestEventService, IngestServiceError},
};

/// `IngestService<T>` is a generic implementation of the `IngestEventService`
/// that can use any corresponding `IngestEventRepository` to carry out save
/// actions on `IngestEvent` structs
#[derive(Clone, Debug)]
pub struct IngestService<T>
where
    T: IngestEventRepository + std::fmt::Debug,
{
    pub ingest_event_repository: T,
}

impl<T> IngestService<T>
where
    T: IngestEventRepository + std::fmt::Debug,
{
    /// `IngestService<T>` constructor
    pub fn new(ingest_event_repository: T) -> Self {
        Self {
            ingest_event_repository,
        }
    }
}

impl<T> IngestEventService for IngestService<T>
where
    T: IngestEventRepository + std::fmt::Debug,
{
    /// `IngestService` implementation of the `save` method that is used to
    /// persist a `Vec` of `<IngestEvent>` to the underlying
    /// `IngestEventRepository`
    #[instrument]
    async fn save(
        &self,
        events: Vec<crate::domain::model::ingest_event::IngestEvent>,
    ) -> Result<IngestActionSummary, IngestServiceError> {
        if events.is_empty() {
            return Err(IngestServiceError::InvalidRequest);
        }
        self.ingest_event_repository
            .save(events)
            .await
            .map_err(|e| match e {
                IngestRepositoryError::InvalidRequest => IngestServiceError::InvalidRequest,
                IngestRepositoryError::Conversion => e.into(),
                IngestRepositoryError::Repository => e.into(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::Uuid;

    use crate::domain::{
        model::{
            ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary},
            ingest_event::{ApiKey, IngestEvent, IngestEventSource, Site, VisitorEvent},
        },
        repository::ingest_event_repository::{
            test::MockIngestEventRepository, IngestRepositoryError,
        },
        service::ingest_event_service::{IngestEventService, IngestServiceError},
    };

    use super::IngestService;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_save() {
        let uuid_now = Uuid::now_v7();

        // Valid success case
        let mock_success_repo = MockIngestEventRepository {
            save_result: Ok(IngestActionSummary::Save(IngestEventSaveSummary {
                event_count: 1,
            })),
            event_source_result: Ok(HashSet::from([IngestEventSource::new(
                ApiKey::new("abc-123"),
                Site::new("test.com"),
            )])),
        };
        let test_success_service = IngestService {
            ingest_event_repository: mock_success_repo,
        };
        let test_events: Vec<IngestEvent> = vec![IngestEvent::Visitor(
            VisitorEvent::try_new(ApiKey::new("abc_123"), Site::new("test.com"), uuid_now).unwrap(),
        )];
        let Ok(IngestActionSummary::Save(save_success_result)) =
            test_success_service.save(test_events).await
        else {
            panic!("Expected to successfully save event with mock");
        };
        assert_eq!(
            save_success_result.event_count, 1,
            "Expected to have 1 result saved in mock"
        );

        // Invalid request with empty vec failure case
        let test_invalid_events: Vec<IngestEvent> = Vec::new();
        let Err(invalid_err) = test_success_service.save(test_invalid_events).await else {
            panic!("Expected to an error from submitting empty event request");
        };
        assert_eq!(
            invalid_err,
            IngestServiceError::InvalidRequest,
            "Expected InvalidRequest error"
        );

        // Valid request with repository returning error
        let mock_err_repo = MockIngestEventRepository {
            save_result: Err(IngestRepositoryError::Repository),
            event_source_result: Err(IngestRepositoryError::Repository),
        };
        let test_err_service = IngestService {
            ingest_event_repository: mock_err_repo,
        };
        let test_events: Vec<IngestEvent> = vec![IngestEvent::Visitor(
            VisitorEvent::try_new(ApiKey::new("abc_123"), Site::new("test.com"), uuid_now).unwrap(),
        )];
        let Err(save_err_result) = test_err_service.save(test_events).await else {
            panic!("Expected to fail save event with mock");
        };
        assert_eq!(
            save_err_result,
            IngestServiceError::Repository(IngestRepositoryError::Repository),
            "Expected to encounter IngestRepositoryError::Repository error"
        );
    }
}
