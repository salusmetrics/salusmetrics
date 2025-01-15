use axum::response::IntoResponse;
use http::StatusCode;

use crate::domain::model::ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary};

#[derive(Debug, Clone)]
pub struct ClientEventSaveSummary {
    pub event_count: usize,
}

impl From<IngestEventSaveSummary> for ClientEventSaveSummary {
    fn from(value: IngestEventSaveSummary) -> Self {
        Self {
            event_count: value.event_count,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientEventActionSummary {
    Save(ClientEventSaveSummary),
}

impl From<IngestActionSummary> for ClientEventActionSummary {
    fn from(value: IngestActionSummary) -> Self {
        match value {
            IngestActionSummary::Save(save_summary) => Self::Save(save_summary.into()),
        }
    }
}

impl IntoResponse for ClientEventActionSummary {
    fn into_response(self) -> axum::response::Response {
        match self {
            ClientEventActionSummary::Save(_) => StatusCode::CREATED.into_response(),
        }
    }
}
