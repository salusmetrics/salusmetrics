use std::sync::Arc;

use crate::domain::service::ingest_event_service::IngestEventService;

/// `IngestApplicationState` is the Axum state that is required for all
/// handlers for the HTTP API for Ingestion. This generic implementation
/// requires an `IngestEventService` that is used for saving incoming events
/// to the data store.
#[derive(Debug, Clone)]
pub struct IngestApplicationState<I: IngestEventService> {
    pub ingest_service: Arc<I>,
}

impl<I: IngestEventService> IngestApplicationState<I> {
    /// `IngestApplicationState` constructor that takes an `IngestEventService`
    /// as the sole argument
    pub fn new(ingest_service: I) -> Self {
        Self {
            ingest_service: Arc::new(ingest_service),
        }
    }
}
