use thiserror::Error;

use crate::domain::{
    model::{ingest_action_summary::IngestActionSummary, ingest_event::IngestEvent},
    repository::ingest_event_repository::IngestRepositoryError,
};

/// Represent potential error cases for an IngestEventService action
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestServiceError {
    #[error("Error saving IngestEvent")]
    Save(#[from] IngestRepositoryError),
}

pub trait IngestEventService: 'static + Send + Sync {
    async fn save(
        &self,
        events: Vec<IngestEvent>,
    ) -> Result<IngestActionSummary, IngestServiceError>;
}
