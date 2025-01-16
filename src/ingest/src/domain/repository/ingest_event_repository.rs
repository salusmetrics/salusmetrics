use std::future::Future;

use thiserror::Error;

use crate::domain::model::{ingest_action_summary::IngestActionSummary, ingest_event::IngestEvent};

/// `IngestRepositoryError` represents potential error cases for an
/// `IngestEventRepository` action
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestRepositoryError {
    /// The `IngestEvent` could not be properly translated into the format or
    /// limitations of the underlying repository record type.
    #[error("Error translating IngestEvent into valid repository record")]
    Conversion,
    /// The underlying specific implementation of the repository returned
    /// an error
    #[error("Error persisting IngestEvent")]
    Repository,
}

/// `IngestEventRepository` is a repository trait that specifies how
/// `IngestEvent` should be handled.
pub trait IngestEventRepository: 'static + Clone + Send + Sync {
    /// `save` a given `Vec` of `IngestEvent` structs
    fn save(
        &self,
        events: Vec<IngestEvent>,
    ) -> impl Future<Output = Result<IngestActionSummary, IngestRepositoryError>> + Send;
}
