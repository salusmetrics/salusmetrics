use std::collections::HashSet;
use std::sync::Arc;

use clickhouse::Client;
use tracing::instrument;

use crate::domain::model::ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary};
use crate::domain::model::ingest_event::{IngestEvent, IngestEventSource};
use crate::domain::repository::ingest_event_repository::{
    IngestEventRepository, IngestRepositoryError,
};

use super::clickhouse_event_record::ClickhouseEventRecord;
use super::clickhouse_source_record::ClickhouseSourceRecord;

/// `ClickhouseIngestRepository` is an implementation of the
/// `IngestEventRepository` trait that utilizes ClickHouse as the back end.
/// Crucially, all event types are saved into ClickHouse in the same table,
/// `EVENT` that has a NULL table engine and merely acts as a common place
/// from which to base materialized views which subsequently populate the
/// specific type tables, which are also monitored by other materialized views
/// that then derive aggregate data for reporting.
#[derive(Clone)]
pub struct ClickhouseIngestRepository {
    metrics_db_client: Client,
    event_sources: Arc<HashSet<IngestEventSource>>,
}

impl std::fmt::Debug for ClickhouseIngestRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickhouseIngestRepository").finish()
    }
}

impl ClickhouseIngestRepository {
    pub async fn try_new(metrics_db_client: Client) -> Result<Self, IngestRepositoryError> {
        let sources: HashSet<IngestEventSource> = retrieve_event_sources(metrics_db_client.clone())
            .await?
            .iter()
            .map(IngestEventSource::from)
            .collect();
        Ok(Self {
            metrics_db_client,
            event_sources: Arc::new(sources),
        })
    }
}

impl IngestEventRepository for ClickhouseIngestRepository {
    /// `save` method for ClickHouse puts all events into a common table called
    /// `EVENT` which is then used to populate all other metrics tables.
    #[instrument]
    async fn save(
        &self,
        events: Vec<IngestEvent>,
    ) -> Result<IngestActionSummary, IngestRepositoryError> {
        if events.is_empty() {
            return Err(IngestRepositoryError::InvalidRequest);
        }
        let mut records: Vec<ClickhouseEventRecord> = Vec::with_capacity(events.len());
        for event in events.iter() {
            match event {
                IngestEvent::Visitor(ref evt) => {
                    if !self.event_sources.contains(&IngestEventSource::from(&evt)) {
                        return Err(IngestRepositoryError::InvalidRequest);
                    }
                }
                IngestEvent::Session(ref evt) => {
                    if !self.event_sources.contains(&IngestEventSource::from(&evt)) {
                        return Err(IngestRepositoryError::InvalidRequest);
                    }
                }
                IngestEvent::Section(ref evt) => {
                    if !self.event_sources.contains(&IngestEventSource::from(&evt)) {
                        return Err(IngestRepositoryError::InvalidRequest);
                    }
                }
                IngestEvent::Click(ref evt) => {
                    if !self.event_sources.contains(&IngestEventSource::from(&evt)) {
                        return Err(IngestRepositoryError::InvalidRequest);
                    }
                }
            }
            records.push(ClickhouseEventRecord::try_from(event)?);
        }

        let mut insert = self
            .metrics_db_client
            .insert::<ClickhouseEventRecord>("EVENT")
            .map_err(|e| {
                tracing::error!("Encountered error initiating ClickHouse Insert: {e}");
                IngestRepositoryError::Repository
            })?;

        for record in records.iter() {
            insert.write(record).await.map_err(|e| {
                tracing::error!("Encountered error inserting records: {e}");
                IngestRepositoryError::Repository
            })?;
        }

        insert.end().await.map_err(|e| {
            tracing::error!("Encountered error ending insert: {e}");
            IngestRepositoryError::Repository
        })?;

        Ok(IngestActionSummary::Save(IngestEventSaveSummary {
            event_count: records.len(),
        }))
    }

    async fn event_sources(&self) -> Result<HashSet<IngestEventSource>, IngestRepositoryError> {
        Ok(self.event_sources.iter().map(|es| es.to_owned()).collect())
    }
}

async fn retrieve_event_sources(
    client: Client,
) -> Result<Vec<ClickhouseSourceRecord>, IngestRepositoryError> {
    client
        .query("SELECT api_key, site FROM API_KEY")
        .fetch_all::<ClickhouseSourceRecord>()
        .await
        .map_err(|e| {
            tracing::error!("Encountered error fetching event source records {e}. This is likely due to connection problems with Clickhouse.");
            IngestRepositoryError::Repository
        })
}

#[cfg(test)]
mod tests {
    use clickhouse::{test, Client};
    use uuid::Uuid;

    use super::*;
    use crate::domain::model::ingest_event::{ApiKey, Site, VisitorEvent};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_save() {
        let mock_sources = Vec::from([ClickhouseSourceRecord::new("abc-123", "test.com")]);
        let mock = test::Mock::new();
        mock.add(test::handlers::provide(mock_sources));
        let recording = mock.add(test::handlers::record());
        let mock_client = Client::default().with_url(mock.url());
        let test_repository = ClickhouseIngestRepository::try_new(mock_client)
            .await
            .unwrap();

        let uuid_now = Uuid::now_v7();

        // Valid test with single Visitor event
        let valid_test_events: Vec<IngestEvent> = vec![IngestEvent::Visitor(
            VisitorEvent::try_new(ApiKey::new("abc-123"), Site::new("test.com"), uuid_now).unwrap(),
        )];
        let valid_test_records: Vec<ClickhouseEventRecord> = valid_test_events
            .iter()
            .map(|ev| ev.try_into().unwrap())
            .collect();
        let Ok(IngestActionSummary::Save(save_summary)) =
            test_repository.save(valid_test_events).await
        else {
            panic!("Expected action save summary to be returned");
        };
        assert_eq!(
            save_summary.event_count, 1,
            "Expected to have one record saved"
        );
        let recorded: Vec<ClickhouseEventRecord> = recording.collect().await;
        assert_eq!(
            valid_test_records, recorded,
            "Expected mock save to match test records"
        );

        // Invalid test with empty Vec of events
        let invalid_test_events: Vec<IngestEvent> = Vec::new();
        let Err(error) = test_repository.save(invalid_test_events).await else {
            panic!("Expected an error when attempting to save empty vec");
        };
        assert_eq!(
            error,
            IngestRepositoryError::InvalidRequest,
            "Expected error to be of type InvalidRequest for save of empty vec"
        );

        // Test request for site with invalid api_key
        let invalid_api_key_events: Vec<IngestEvent> = vec![IngestEvent::Visitor(
            VisitorEvent::try_new(ApiKey::new("abc-123-xyz"), Site::new("test.com"), uuid_now)
                .unwrap(),
        )];

        let Err(error) = test_repository.save(invalid_api_key_events).await else {
            panic!("Expected an error when attempting to save with wrong api_key");
        };
        assert_eq!(
            error,
            IngestRepositoryError::InvalidRequest,
            "Expected error to be of type InvalidRequest for save of invalid api_key"
        );
    }
}
