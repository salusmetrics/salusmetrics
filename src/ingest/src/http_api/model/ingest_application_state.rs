use std::sync::Arc;

use crate::domain::service::ingest_event_service::IngestEventService;

#[derive(Debug, Clone)]
pub struct IngestApplicationState<I: IngestEventService> {
    pub ingest_service: Arc<I>,
}

impl<I: IngestEventService> IngestApplicationState<I> {
    pub fn new(ingest_service: I) -> Self {
        Self {
            ingest_service: Arc::new(ingest_service),
        }
    }
}
