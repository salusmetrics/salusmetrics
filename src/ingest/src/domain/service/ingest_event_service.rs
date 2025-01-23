use std::{collections::HashSet, future::Future};

use thiserror::Error;

use crate::domain::{
    model::{
        ingest_action_summary::IngestActionSummary,
        ingest_event::{IngestEvent, IngestEventSource},
    },
    repository::ingest_event_repository::IngestRepositoryError,
};

/// `IngestServiceError` represents potential error cases for an
/// `IngestEventService` action.
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestServiceError {
    /// `InvalidRequest` represents a request that didn't match rules
    /// like saving an empty `Vec` of `IngestEvent`
    #[error("Invalid request")]
    InvalidRequest,
    /// `Repository` allows underlying `IngestRepositoryError` errors to be
    /// handled at the service level
    #[error("Error handling IngestEvent")]
    Repository(#[from] IngestRepositoryError),
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

    /// `event_sources` returns a HashSet of `IngestEventSource` structs that
    /// are configured in the underlying datasource. This represents the full
    /// set of `api_key` / `site` combinations that this server will accept
    fn event_sources(
        &self,
    ) -> impl Future<Output = Result<HashSet<IngestEventSource>, IngestServiceError>> + Send;
}
