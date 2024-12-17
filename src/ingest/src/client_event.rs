use clickhouse::Row;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub const MAX_DURATION_BEFORE_PRESENT: Duration = Duration::HOUR;
pub const MAX_DURATION_AFTER_PRESENT: Duration = Duration::minutes(5);

/// Type of analytics event - maps to ClickHouse Enum8 with identical values
#[derive(Debug, Deserialize_repr, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Clone)]
#[repr(u8)]
pub enum IngestEventType {
    Visitor = 1,
    Session = 2,
    Section = 3,
    Click = 5,
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
    pub event_type: IngestEventType,
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    pub attrs: Vec<(String, String)>,
}

/// Represents the data that will actually be inserted into ClickHouse in the
/// SALUS_METRICS.EVENT table. Expected usages is to call
/// InsertIngestEvent::from_ingest_event on a IngestEvent that has been
/// received.
#[derive(Debug, Row, Deserialize, Serialize, Clone)]
pub struct EventRow {
    api_id: String,
    site: String,
    event_type: IngestEventType,
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    ts: OffsetDateTime,
    attrs: Vec<(String, String)>,
}

/// Represent potential error cases for an IngestEvent, either due to data
/// correctness issues or due to system availability problems.
#[derive(Clone, Error, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum IngestError {
    #[error("Timestamp from UUID beyond acceptable range for new event")]
    TimestampOutOfRange,
    #[error("UUID version mismatch - must be UUIDv7")]
    UuidVersion,
    #[error("UUID timestamp conversion error")]
    UuidTimestampConversion,
}

/// Function that attempts to derive a datetime from the supplied UUID and
/// return an OffsetDateTime.
///
/// The supplied UUID should be of type UUID v7. Any other type should fail.
/// Additionally, the difference in handling of UNIX timestamps can cause
/// errors if the u64 cannot be properly converted to i64 or if the value
/// is out of the component range of the OffsetDateTime crate.
fn try_uuid_datetime(uuid: &Uuid) -> Result<OffsetDateTime, IngestError> {
    let (sec, _) = uuid
        .get_timestamp()
        .ok_or(IngestError::UuidVersion)?
        .to_unix();
    let sec_i64 = i64::try_from(sec).map_err(|_| IngestError::UuidTimestampConversion)?;
    let offset = OffsetDateTime::from_unix_timestamp(sec_i64)
        .map_err(|_| IngestError::UuidTimestampConversion)?;
    Ok(offset)
}

impl EventRow {
    #[tracing::instrument]
    pub fn from_ingest_event(value: ClientEvent, site: &str) -> Result<Self, IngestError> {
        // Determine if the UUID is a proper v7 and if the date is close to now
        let uuid_datetime = Self::verify_datetime_range(try_uuid_datetime(&value.id)?)?;
        Ok(EventRow {
            api_id: value.api_id,
            site: site.to_string(),
            event_type: value.event_type,
            id: value.id,
            ts: uuid_datetime,
            attrs: value.attrs,
        })
    }

    /// Function to check whether the submitted even has a timestamp which falls
    /// within the max and min duration from now
    ///
    /// All events must be no earlier than now - MAX_BEFORE_PRESENT and no later
    /// than now + MAX_AFTER_PRESENT. Any value outside of this range will result
    /// in a TimestampOutOfRange error
    fn verify_datetime_range(datetime: OffsetDateTime) -> Result<OffsetDateTime, IngestError> {
        let now = OffsetDateTime::now_utc();
        if (datetime < (now - MAX_DURATION_BEFORE_PRESENT))
            || (datetime > (now + MAX_DURATION_AFTER_PRESENT))
        {
            Err(IngestError::TimestampOutOfRange)
        } else {
            Ok(datetime)
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Timestamp;

    use super::*;

    const UUID_V4_STR: &str = "4e2abe52-5e86-4023-9f8b-34eba8d2cc59";
    const API_KEY_STR: &str = "123_456_789";
    const SITE: &str = "test.com";

    #[test]
    fn test_try_uuid_datetime() {
        //  Test a valid case with a v7 UUID from now
        let uuid = Uuid::now_v7();
        let odt = try_uuid_datetime(&uuid);
        assert!(odt.is_ok());

        // Test invalid type that is v4
        let uuid = Uuid::parse_str(UUID_V4_STR).unwrap();
        assert_eq!(
            try_uuid_datetime(&uuid).unwrap_err(),
            IngestError::UuidVersion
        );
    }

    #[test]
    fn test_verify_datetime_range() {
        let valid_now = OffsetDateTime::now_utc();
        let valid_early = valid_now - Duration::minutes(30);
        let valid_late = valid_now + Duration::minutes(2);
        let invalid_early = valid_now - Duration::minutes(70);
        let invalid_late = valid_now + Duration::minutes(20);

        // Should succeed with no panic
        EventRow::verify_datetime_range(valid_now).unwrap();
        EventRow::verify_datetime_range(valid_early).unwrap();
        EventRow::verify_datetime_range(valid_late).unwrap();
        // Should return an Err of type IngestError::TimestampOutOfRange
        assert_eq!(
            EventRow::verify_datetime_range(invalid_early).unwrap_err(),
            IngestError::TimestampOutOfRange
        );
        assert_eq!(
            EventRow::verify_datetime_range(invalid_late).unwrap_err(),
            IngestError::TimestampOutOfRange
        );
    }

    #[test]
    fn test_from_ingest_event() {
        let uuid_now = Uuid::now_v7();
        let (ts_now, _) = uuid_now.get_timestamp().unwrap().to_unix();
        let valid_ingest_event = ClientEvent {
            api_id: API_KEY_STR.to_string(),
            event_type: IngestEventType::Visitor,
            id: uuid_now,
            attrs: Vec::new(),
        };

        let invalid_ingest_event_type = ClientEvent {
            id: Uuid::parse_str(UUID_V4_STR).unwrap(),
            ..valid_ingest_event.clone()
        };

        let invalid_ingest_event_early = ClientEvent {
            id: Uuid::new_v7(Timestamp::from_unix_time(ts_now - 3601, 0, 0, 8)),
            ..valid_ingest_event.clone()
        };

        let invalid_ingest_event_late = ClientEvent {
            id: Uuid::new_v7(Timestamp::from_unix_time(ts_now + 301, 0, 0, 8)),
            ..valid_ingest_event.clone()
        };

        EventRow::from_ingest_event(valid_ingest_event, SITE).unwrap();
        assert_eq!(
            EventRow::from_ingest_event(invalid_ingest_event_type, SITE).unwrap_err(),
            IngestError::UuidVersion
        );
        assert_eq!(
            EventRow::from_ingest_event(invalid_ingest_event_early, SITE).unwrap_err(),
            IngestError::TimestampOutOfRange
        );
        assert_eq!(
            EventRow::from_ingest_event(invalid_ingest_event_late, SITE).unwrap_err(),
            IngestError::TimestampOutOfRange
        );
    }
}
