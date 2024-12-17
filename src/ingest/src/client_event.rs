use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of analytics event submitted by client
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ClientEventType {
    Visitor,
    Session,
    Section,
    Click,
}

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
    pub api_id: String,
    pub event_type: ClientEventType,
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    pub attrs: Vec<(String, String)>,
}
