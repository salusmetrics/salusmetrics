use thiserror::Error;

use crate::domain::model::{ingest_action_summary::IngestActionSummary, ingest_event::IngestEvent};

/// Represent potential error cases for an IngestEventRepository action
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestRepositoryError {
    #[error("Error translating IngestEvent into valid repository record")]
    Conversion,
    #[error("Error persisting IngestEvent")]
    Repository,
}

pub trait IngestEventRepository: 'static + Clone + Send + Sync {
    async fn save(
        &self,
        events: Vec<IngestEvent>,
    ) -> Result<IngestActionSummary, IngestRepositoryError>;
}
