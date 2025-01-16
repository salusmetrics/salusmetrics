use axum::response::IntoResponse;
use http::StatusCode;

use crate::domain::model::ingest_action_summary::{IngestActionSummary, IngestEventSaveSummary};

/// `ClientEventSaveSummary` provides a struct outside of the domain to
/// encapsulate the results from the service layer.
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

/// `ClientEventActionSummary` provides a clean separation from the domain
/// objects and can implement Axum `IntoResponse`
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

/// `ClientEventActionSummary` should be able to be returned from handler
/// functions so that there is a clean
/// `Result<ClientEventActionSummary, ClientEventRequestError>` return
/// signature for the handlers. But, we don't need to actually provide data
/// back to the client, so this type simply maps to a HTTP 201 Created response
impl IntoResponse for ClientEventActionSummary {
    fn into_response(self) -> axum::response::Response {
        match self {
            ClientEventActionSummary::Save(_) => StatusCode::CREATED.into_response(),
        }
    }
}
