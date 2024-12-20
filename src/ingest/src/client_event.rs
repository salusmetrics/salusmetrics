use http::header;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ingest_error::IngestError;

/// Type of analytics event submitted by client
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ClientEventType {
    Visitor,
    Session,
    Section,
    Click,
}

/// Newtype wrapper for handling api_key
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKey(pub String);

/// Newtype wrapper for handling site
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site(pub String);

/// Represents IngestEvent that an external, untrusted client publishes
///
/// IngestEvent is general across all types of events from new visitors down to
/// page actions. Handled by including IngestEventType and generic attrs Vec of
/// tuples to represend necessary data across type. Critically, the id must be
/// a UUID v7 with the datetime portion of this id extracted to represent the
/// event time. If this time is not within a certain span of now, the event is
/// rejected.
///
/// The expectation for this event type is that some data is explicitly placed
/// in attrs by the client, but other data will be added by the server side.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientEvent {
    pub event_type: ClientEventType,
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    pub attrs: Option<Vec<(String, String)>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventHeaders {
    pub api_key: ApiKey,
    pub site: Site,
}

impl TryFrom<&HeaderMap> for EventHeaders {
    type Error = IngestError;

    fn try_from(value: &HeaderMap) -> Result<Self, Self::Error> {
        let site = Site(
            value
                .get(header::REFERER)
                .ok_or(IngestError::Site)?
                .to_str()
                .map_err(|_| IngestError::Site)?
                .to_string(),
        );
        Ok(EventHeaders {
            api_key: ApiKey("123".to_string()),
            site,
        })
    }
}
