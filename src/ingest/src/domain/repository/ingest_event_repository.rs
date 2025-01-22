use std::{collections::HashSet, future::Future};

use thiserror::Error;

use crate::domain::model::{
    ingest_action_summary::IngestActionSummary,
    ingest_event::{IngestEvent, IngestEventSource},
};

/// `IngestRepositoryError` represents potential error cases for an
/// `IngestEventRepository` action
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestRepositoryError {
    /// The `IngestEvent` could not be properly translated into the format or
    /// limitations of the underlying repository record type.
    #[error("Error translating IngestEvent into valid repository record")]
    Conversion,
    /// The request was invalid, due to empty list of events or other cause
    #[error("Invalid request")]
    InvalidRequest,
    /// The underlying specific implementation of the repository returned
    /// an error
    #[error("Error persisting IngestEvent")]
    Repository,
}

/// `IngestEventRepository` is a repository trait that specifies how
/// `IngestEvent` and related domain models should be persisted and queried.
pub trait IngestEventRepository: 'static + Clone + Send + Sync {
    /// `save` a given `Vec` of `IngestEvent` structs
    fn save(
        &self,
        events: Vec<IngestEvent>,
    ) -> impl Future<Output = Result<IngestActionSummary, IngestRepositoryError>> + Send;

    /// `event_sources` attemots to return a HashSet of allowed
    /// `IngestEventSource` structs that the underlyind data source is
    /// configured to handle.
    fn event_sources(
        &self,
    ) -> impl Future<Output = Result<HashSet<IngestEventSource>, IngestRepositoryError>> + Send;
}

/// Provide a mock for the `IngestEventRepository` trait to be used in other
/// code that needs to unit test
#[cfg(test)]
pub(crate) mod test {
    use crate::domain::model::{
        ingest_action_summary::IngestEventSaveSummary,
        ingest_event::{ApiKey, Site},
    };

    use super::*;

    #[derive(Clone, Debug)]
    pub(crate) struct MockIngestEventRepository {
        pub(crate) save_result: Result<IngestActionSummary, IngestRepositoryError>,
        pub(crate) event_source_result: Result<HashSet<IngestEventSource>, IngestRepositoryError>,
    }

    impl IngestEventRepository for MockIngestEventRepository {
        async fn save(
            &self,
            _: Vec<IngestEvent>,
        ) -> Result<IngestActionSummary, IngestRepositoryError> {
            self.save_result.clone()
        }
        async fn event_sources(&self) -> Result<HashSet<IngestEventSource>, IngestRepositoryError> {
            self.event_source_result.clone()
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_repository() {
        let mock_success_repo = MockIngestEventRepository {
            save_result: Ok(IngestActionSummary::Save(IngestEventSaveSummary {
                event_count: 5,
            })),
            event_source_result: Ok(HashSet::from([IngestEventSource::new(
                ApiKey::new("abc-123"),
                Site::new("test.com"),
            )])),
        };
        let mock_save_success_result = mock_success_repo.save(Vec::new()).await.unwrap();
        match mock_save_success_result {
            IngestActionSummary::Save(save_summary) => assert_eq!(
                save_summary.event_count, 5,
                "Expected to receive 5 event count for this valid test"
            ),
        }

        let mock_source_success_result = mock_success_repo.event_sources().await.unwrap();
        assert!(
            mock_source_success_result.contains(&IngestEventSource::new(
                ApiKey::new("abc-123"),
                Site::new("test.com")
            )),
            "Expected IngestEventSource to be in returned result for this repo mock"
        );

        let mock_failure_repo = MockIngestEventRepository {
            save_result: Err(IngestRepositoryError::Repository),
            event_source_result: Err(IngestRepositoryError::Repository),
        };
        let mock_save_failure_result = mock_failure_repo.save(Vec::new()).await.unwrap_err();
        assert_eq!(mock_save_failure_result, IngestRepositoryError::Repository);
        let mock_source_failure_result = mock_failure_repo.event_sources().await.unwrap_err();
        assert_eq!(
            mock_source_failure_result,
            IngestRepositoryError::Repository
        );
    }
}
