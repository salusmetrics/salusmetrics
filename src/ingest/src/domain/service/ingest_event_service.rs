use std::future::Future;

use thiserror::Error;

use crate::domain::{
    model::{ingest_action_summary::IngestActionSummary, ingest_event::IngestEvent},
    repository::ingest_event_repository::IngestRepositoryError,
};

/// `IngestServiceError` represents potential error cases for an
/// `IngestEventService` action.
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestServiceError {
    /// `Save` allows underlying `IngestRepositoryError` errors to be
    /// handled at the service level
    #[error("Error saving IngestEvent")]
    Save(#[from] IngestRepositoryError),
}

/// `IngestEventService` trait provides the service interface for
/// various `IngestEvent` actions
pub trait IngestEventService: 'static + Send + Sync {
    /// `save` a `Vec` of `IngestEvents` to the associated
    /// `IngestEventRepository`
    fn save(
        &self,
        events: Vec<IngestEvent>,
    ) -> impl Future<Output = Result<IngestActionSummary, IngestServiceError>> + Send;
}
