use crate::client_event::{ApiKey, ClientEvent, ClientEventType, Site};
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::OffsetDateTime;
use uuid::Uuid;

/// Type of analytics event - maps to ClickHouse Enum8 with identical values
/// See definition of table `SALUS_METRICS.EVENT` and field `event_type`
#[derive(Debug, Deserialize_repr, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Clone)]
#[repr(u8)]
pub enum EventRecordType {
    Visitor = 1,
    Session = 2,
    Section = 3,
    Click = 4,
}

/// Represents the data that will actually be inserted into ClickHouse in the
/// SALUS_METRICS.EVENT table. Expected usages is to call
/// InsertIngestEvent::from_ingest_event on a IngestEvent that has been
/// received.
#[derive(Debug, Row, Deserialize, Serialize, Clone)]
pub struct EventRecord {
    api_key: ApiKey,
    site: Site,
    event_type: EventRecordType,
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    ts: OffsetDateTime,
    attrs: Vec<(String, String)>,
}

impl From<ClientEventType> for EventRecordType {
    fn from(value: ClientEventType) -> Self {
        match value {
            ClientEventType::Visitor => Self::Visitor,
            ClientEventType::Session => Self::Session,
            ClientEventType::Section => Self::Section,
            ClientEventType::Click => Self::Click,
        }
    }
}

impl From<&ClientEvent> for EventRecord {
    fn from(event: &ClientEvent) -> Self {
        EventRecord {
            api_key: event.api_key(),
            site: event.site(),
            event_type: event.event_type().into(),
            id: event.id(),
            ts: event.ts(),
            attrs: event.attrs().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This is very important in order to keep the mapping in ClickHouse in
    /// line with this library
    #[test]
    fn test_event_record_type_discriminant() {
        let visitor_discriminant = EventRecordType::Visitor as u32;
        assert_eq!(
            visitor_discriminant, 1,
            "EventRecordType::Visitor discriminant does not match expected value"
        );

        let session_discriminant = EventRecordType::Session as u32;
        assert_eq!(
            session_discriminant, 2,
            "EventRecordType::Session discriminant does not match expected value"
        );

        let section_discriminant = EventRecordType::Section as u32;
        assert_eq!(
            section_discriminant, 3,
            "EventRecordType::Section discriminant does not match expected value"
        );

        let click_discriminant = EventRecordType::Click as u32;
        assert_eq!(
            click_discriminant, 4,
            "EventRecordType::Click discriminant does not match expected value"
        );
    }

    #[test]
    fn test_try_from_client_event() {
        let uuid_now = Uuid::now_v7();
        let Ok(valid_ingest_event) = ClientEvent::try_new(
            ApiKey("abc-124".to_owned()),
            Site("http://salusmetrics.com".to_owned()),
            ClientEventType::Visitor,
            uuid_now,
            Some(Vec::new()),
        ) else {
            panic!("Expected valid ingest event")
        };
        let _ = EventRecord::from(&valid_ingest_event);
    }
}
