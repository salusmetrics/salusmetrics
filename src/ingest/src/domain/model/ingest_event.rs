use std::marker::PhantomData;

use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::model::util::{is_ts_within_ingest_range, try_uuid_datetime};

/// Represent potential error cases for an IngestEvent, either due to data
/// correctness issues or due to system availability problems.
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestEventError {
    #[error("Timestamp from UUID beyond acceptable range for new event")]
    TimestampOutOfRange,
    #[error("UUID version mismatch - must be UUIDv7")]
    UuidVersion,
    #[error("UUID timestamp conversion error")]
    UuidTimestampConversion,
}

/// `IngestEvent` is the domain model for all metrics that the system is able
/// to handle. This internal representation takes care of doing the validation
/// and other domain concerns for events.
#[derive(Debug, Clone)]
pub enum IngestEvent {
    Visitor(VisitorEvent),
    Session(SessionEvent),
    Section(SectionEvent),
    Click(ClickEvent),
}

/// `ApiKey` newtype wrapper for the api_key string
#[derive(Debug, Clone)]
pub struct ApiKey(pub String);

impl ApiKey {
    /// `ApiKey` constructor
    pub fn new(key: impl AsRef<str>) -> Self {
        Self(key.as_ref().to_owned())
    }
}

/// `Site` newtype wrapper for the site an event is coming from
#[derive(Debug, Clone)]
pub struct Site(pub String);

impl Site {
    /// `Site` constructor
    pub fn new(site: impl AsRef<str>) -> Self {
        Self(site.as_ref().to_owned())
    }
}

/// `IngestEventCore` represents the common fields that all events have like
/// `api_key`, `id` and `ts`.
///
/// Note that the timestamp, `ts` for the event is strictly derived from the
/// `id` field which must be a UUIDv7 or else the construction of this struct
/// will result in an error. Additionally, the associeated timestamp for any
/// given ingestion event must be within a specified duration of now, or else
/// an error will be returned during attempted construction.
#[derive(Debug, Clone)]
pub struct IngestEventCore<T> {
    pub api_key: ApiKey,
    pub site: Site,
    pub id: Uuid,
    pub ts: OffsetDateTime,
    _event_type: PhantomData<T>,
}

impl<T> IngestEventCore<T> {
    /// `IngestEventCore` constructor
    pub fn try_new(api_key: ApiKey, site: Site, id: Uuid) -> Result<Self, IngestEventError> {
        let ts = try_uuid_datetime(&id)?;

        if is_ts_within_ingest_range(&ts) {
            Ok(Self {
                api_key,
                site,
                id,
                ts,
                _event_type: PhantomData,
            })
        } else {
            Err(IngestEventError::TimestampOutOfRange)
        }
    }
}

/// `VisitorEvent` represents a an event where an unrecognized user begins to
/// use the site. The `VisitorEvent` gives us a root against which all subsequent
/// events are based.
#[derive(Debug, Clone)]
pub struct VisitorEvent {
    pub core: IngestEventCore<Self>,
}
impl VisitorEvent {
    /// `VisitorEvent` all field constructor
    pub fn try_new(api_key: ApiKey, site: Site, id: Uuid) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(IngestEventCore::try_new(api_key, site, id)?)
    }

    /// `VisitorEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    pub fn try_new_with_core_event(core: IngestEventCore<Self>) -> Result<Self, IngestEventError> {
        Ok(Self { core })
    }
}

#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub core: IngestEventCore<Self>,
    pub parent: Uuid,
}
impl SessionEvent {
    /// `SessionEvent` all field constructor
    pub fn try_new(
        api_key: ApiKey,
        site: Site,
        id: Uuid,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(IngestEventCore::try_new(api_key, site, id)?, parent)
    }

    /// `SessionEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    pub fn try_new_with_core_event(
        core: IngestEventCore<Self>,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Ok(Self { core, parent })
    }
}

#[derive(Debug, Clone)]
pub struct SectionEvent {
    pub core: IngestEventCore<Self>,
    pub parent: Uuid,
}
impl SectionEvent {
    /// `SectionEvent` all field constructor
    pub fn try_new(
        api_key: ApiKey,
        site: Site,
        id: Uuid,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(IngestEventCore::try_new(api_key, site, id)?, parent)
    }

    /// `SectionEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    pub fn try_new_with_core_event(
        core: IngestEventCore<Self>,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Ok(Self { core, parent })
    }
}

#[derive(Debug, Clone)]
pub struct ClickEvent {
    pub core: IngestEventCore<Self>,
    pub parent: Uuid,
}
impl ClickEvent {
    /// `ClickEvent` all field constructor
    pub fn try_new(
        api_key: ApiKey,
        site: Site,
        id: Uuid,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(IngestEventCore::try_new(api_key, site, id)?, parent)
    }

    /// `ClickEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    pub fn try_new_with_core_event(
        core: IngestEventCore<Self>,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Ok(Self { core, parent })
    }
}

#[cfg(test)]
mod tests {
    use uuid::{Timestamp, Uuid};

    use super::*;

    const UUID_V4_STR: &str = "4e2abe52-5e86-4023-9f8b-34eba8d2cc59";
    const API_KEY_STR: &str = "123_456_789";
    const SITE: &str = "test.com";

    #[test]
    fn test_try_new_events() {
        let uuid_now = Uuid::now_v7();
        let (ts_now, _) = uuid_now.get_timestamp().unwrap().to_unix();
        let test_core: Result<IngestEventCore<VisitorEvent>, IngestEventError> =
            IngestEventCore::try_new(ApiKey::new(API_KEY_STR), Site::new(SITE), uuid_now);
        let Ok(_) = test_core else {
            panic!("Expected valid IngestEventCore");
        };

        let invalid_ingest_uuid_type: Result<IngestEventCore<SessionEvent>, IngestEventError> =
            IngestEventCore::try_new(
                ApiKey::new(API_KEY_STR),
                Site::new(SITE),
                Uuid::parse_str(UUID_V4_STR).unwrap(),
            );
        assert_eq!(
            invalid_ingest_uuid_type.unwrap_err(),
            IngestEventError::UuidVersion
        );

        let invalid_ingest_event_early: Result<IngestEventCore<SectionEvent>, IngestEventError> =
            IngestEventCore::try_new(
                ApiKey::new(API_KEY_STR),
                Site::new(SITE),
                Uuid::new_v7(Timestamp::from_unix_time(ts_now - 3601, 0, 0, 8)),
            );
        assert_eq!(
            invalid_ingest_event_early.unwrap_err(),
            IngestEventError::TimestampOutOfRange
        );

        let invalid_ingest_event_late: Result<IngestEventCore<ClickEvent>, IngestEventError> =
            IngestEventCore::try_new(
                ApiKey::new(API_KEY_STR),
                Site::new(SITE),
                Uuid::new_v7(Timestamp::from_unix_time(ts_now + 301, 0, 0, 8)),
            );
        assert_eq!(
            invalid_ingest_event_late.unwrap_err(),
            IngestEventError::TimestampOutOfRange
        );

        let Ok(_) =
            VisitorEvent::try_new(ApiKey::new(API_KEY_STR), Site::new(SITE), Uuid::now_v7())
        else {
            panic!("Expected valid VisitorEvent");
        };

        let Ok(_) = SessionEvent::try_new(
            ApiKey::new(API_KEY_STR),
            Site::new(SITE),
            Uuid::now_v7(),
            Uuid::now_v7(),
        ) else {
            panic!("Expected valid SessionEvent");
        };

        let Ok(_) = SectionEvent::try_new(
            ApiKey::new(API_KEY_STR),
            Site::new(SITE),
            Uuid::now_v7(),
            Uuid::now_v7(),
        ) else {
            panic!("Expected valid SectionEvent");
        };

        let Ok(_) = ClickEvent::try_new(
            ApiKey::new(API_KEY_STR),
            Site::new(SITE),
            Uuid::now_v7(),
            Uuid::now_v7(),
        ) else {
            panic!("Expected valid ClickEvent");
        };
    }
}
