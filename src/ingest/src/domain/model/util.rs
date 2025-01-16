use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::domain::model::ingest_event::IngestEventError;

/// Earliest event the system treats as valid for ingestion relative to now
const MAX_DURATION_BEFORE_PRESENT: Duration = Duration::HOUR;
/// Latest event the system treats as valid for ingestion relative to now
const MAX_DURATION_AFTER_PRESENT: Duration = Duration::minutes(5);

/// Domain functions for Ingest
/// Function that attempts to derive a datetime from the supplied UUID and
/// return an OffsetDateTime.
///
/// The supplied UUID should be of type UUID v7. Any other type should fail.
/// Additionally, the difference in handling of UNIX timestamps can cause
/// errors if the u64 cannot be properly converted to i64 or if the value
/// is out of the component range of the OffsetDateTime crate.
pub(crate) fn try_uuid_datetime(uuid: Uuid) -> Result<OffsetDateTime, IngestEventError> {
    let (sec, _) = uuid
        .get_timestamp()
        .ok_or(IngestEventError::UuidVersion)?
        .to_unix();
    let sec_i64 = i64::try_from(sec).map_err(|_| IngestEventError::UuidTimestampConversion)?;
    let offset = OffsetDateTime::from_unix_timestamp(sec_i64)
        .map_err(|_| IngestEventError::UuidTimestampConversion)?;
    Ok(offset)
}

/// Function to check whether the submitted even has a timestamp which falls
/// within the max and min duration from now
///
/// All events must be no earlier than now - MAX_BEFORE_PRESENT and no later
/// than now + MAX_AFTER_PRESENT.
pub(crate) fn is_ts_within_ingest_range(ts: &OffsetDateTime) -> bool {
    let now = OffsetDateTime::now_utc();
    (ts > &(now - MAX_DURATION_BEFORE_PRESENT)) && (ts < &(now + MAX_DURATION_AFTER_PRESENT))
}

#[cfg(test)]
mod tests {
    use time::{Duration, OffsetDateTime};
    use uuid::Uuid;

    use super::*;
    use crate::domain::model::ingest_event::IngestEventError;

    const UUID_V4_STR: &str = "4e2abe52-5e86-4023-9f8b-34eba8d2cc59";

    #[test]
    fn test_try_uuid_datetime() {
        //  Test a valid case with a v7 UUID from now
        let uuid = Uuid::now_v7();
        let odt = try_uuid_datetime(uuid);
        assert!(odt.is_ok());

        // Test invalid type that is v4
        let uuid = Uuid::parse_str(UUID_V4_STR).unwrap();
        assert_eq!(
            try_uuid_datetime(uuid).unwrap_err(),
            IngestEventError::UuidVersion
        );
    }

    #[test]
    fn test_is_ts_within_ingest_range() {
        let valid_now = OffsetDateTime::now_utc();
        let valid_early = valid_now - Duration::minutes(30);
        let valid_late = valid_now + Duration::minutes(2);
        let invalid_early = valid_now - Duration::minutes(70);
        let invalid_late = valid_now + Duration::minutes(20);

        // Should succeed with no panic
        assert!(
            is_ts_within_ingest_range(&valid_now),
            "Current ts should be valid ingest OffsetDateTime"
        );
        assert!(
            is_ts_within_ingest_range(&valid_early),
            "30 minutes prior should be valid ingest OffsetDateTime"
        );
        assert!(
            is_ts_within_ingest_range(&valid_late),
            "2 minutes after should be vaid ingest OffsetDateTime"
        );
        // Should return an Err of type IngestError::TimestampOutOfRange
        assert!(
            !is_ts_within_ingest_range(&invalid_early),
            "70 minutes prior should be invalid ingest OffsetDateTime"
        );
        assert!(
            !is_ts_within_ingest_range(&invalid_late),
            "20 minutes prior should be invalid ingest OffsetDateTime"
        );
    }
}
