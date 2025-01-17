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
    /// The request was invalid, due to empty list of events or other cause
    #[error("Invalid request")]
    InvalidRequest,
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

/// Provide a mock for the `IngestEventRepository` trait to be used in other
/// code that needs to unit test
#[cfg(test)]
pub(crate) mod test {
    use crate::domain::model::ingest_action_summary::IngestEventSaveSummary;

    use super::*;

    #[derive(Clone, Debug)]
    pub(crate) struct MockIngestEventRepository {
        pub(crate) save_result: Result<IngestActionSummary, IngestRepositoryError>,
    }

    impl IngestEventRepository for MockIngestEventRepository {
        async fn save(
            &self,
            _: Vec<IngestEvent>,
        ) -> Result<IngestActionSummary, IngestRepositoryError> {
            self.save_result.clone()
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_repository() {
        let mock_success_repo = MockIngestEventRepository {
            save_result: Ok(IngestActionSummary::Save(IngestEventSaveSummary {
                event_count: 5,
            })),
        };
        let mock_success_result = mock_success_repo.save(Vec::new()).await.unwrap();
        match mock_success_result {
            IngestActionSummary::Save(save_summary) => assert_eq!(
                save_summary.event_count, 5,
                "Expected to receive 5 event count for this valid test"
            ),
        }

        let mock_failure_repo = MockIngestEventRepository {
            save_result: Err(IngestRepositoryError::Repository),
        };
        let mock_failure_result = mock_failure_repo.save(Vec::new()).await.unwrap_err();
        assert_eq!(mock_failure_result, IngestRepositoryError::Repository);
    }
}
