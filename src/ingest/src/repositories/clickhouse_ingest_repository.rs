use clickhouse::Client;

use crate::domain::model::ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary};
use crate::domain::model::ingest_event::IngestEvent;
use crate::domain::repository::ingest_event_repository::{
    IngestEventRepository, IngestRepositoryError,
};

use super::clickhouse_event_record::ClickhouseEventRecord;

#[derive(Clone)]
pub struct ClickhouseIngestRepository {
    metrics_db_client: Client,
}

impl ClickhouseIngestRepository {
    pub fn new(metrics_db_client: Client) -> Self {
        Self { metrics_db_client }
    }
}

impl IngestEventRepository for ClickhouseIngestRepository {
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
        Ok(IngestActionSummary::Save(IngestEventSaveSummary {
            event_count: records.len(),
        }))
    }
}
