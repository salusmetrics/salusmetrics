use clickhouse::Client;
use tracing::instrument;

use crate::domain::model::ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary};
use crate::domain::model::ingest_event::IngestEvent;
use crate::domain::repository::ingest_event_repository::{
    IngestEventRepository, IngestRepositoryError,
};

use super::clickhouse_event_record::ClickhouseEventRecord;

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
}

impl std::fmt::Debug for ClickhouseIngestRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickhouseIngestRepository").finish()
    }
}

impl ClickhouseIngestRepository {
    pub fn new(metrics_db_client: Client) -> Self {
        Self { metrics_db_client }
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
        let mut records: Vec<ClickhouseEventRecord> = Vec::with_capacity(events.len());
        for event in events.iter() {
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
}
