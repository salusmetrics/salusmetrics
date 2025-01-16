use crate::domain::{
    model::ingest_action_summary::IngestActionSummary,
    repository::ingest_event_repository::IngestEventRepository,
    service::ingest_event_service::{IngestEventService, IngestServiceError},
};

/// `IngestService<T>` is a generic implementation of the `IngestEventService`
/// that can use any corresponding `IngestEventRepository` to carry out save
/// actions on `IngestEvent` structs
#[derive(Clone)]
pub struct IngestService<T>
where
    T: IngestEventRepository,
{
    pub ingest_event_repository: T,
}

impl<T> IngestService<T>
where
    T: IngestEventRepository,
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
    T: IngestEventRepository,
{
    /// `IngestService` implementation of the `save` method that is used to
    /// persist a `Vec` of `<IngestEvent>` to the underlying
    /// `IngestEventRepository`
    async fn save(
        &self,
        events: Vec<crate::domain::model::ingest_event::IngestEvent>,
    ) -> Result<IngestActionSummary, IngestServiceError> {
        self.ingest_event_repository
            .save(events)
            .await
            .map_err(|e| e.into())
    }
}
