use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::model::util::{is_ts_within_ingest_range, try_uuid_datetime};

/// `IngestEventError` represents the  potential domain error cases for
/// `IngestEvent`. This is strictly due to domain rules, not infrastructure
/// related issues.
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum IngestEventError {
    #[error("api_key submitted was empty")]
    ApiKey,
    #[error("site submitted was empty")]
    Site,
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
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ApiKey {
    api_key: String,
}

impl ApiKey {
    /// `ApiKey` constructor
    pub fn new(key: impl AsRef<str>) -> Self {
        Self {
            api_key: key.as_ref().trim().to_owned(),
        }
    }
    /// Provide access to the api_key value
    pub fn value(&self) -> &String {
        &self.api_key
    }
}

/// `Site` newtype wrapper for the site an event is coming from
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Site {
    site: String,
}

impl Site {
    /// `Site` constructor
    pub fn new(site: impl AsRef<str>) -> Self {
        Self {
            site: site.as_ref().trim().to_owned(),
        }
    }
    /// provide access to the site value
    pub fn value(&self) -> &String {
        &self.site
    }
}

/// `IngestEventSource` represents the combination of the api_key and the
/// site into a single entity that is used to check whether or not to continue
/// processing a given event
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IngestEventSource {
    api_key: ApiKey,
    site: Site,
}

impl<T> From<&T> for IngestEventSource
where
    T: CommonEvent,
{
    fn from(value: &T) -> Self {
        Self {
            api_key: value.api_key().to_owned(),
            site: value.site().to_owned(),
        }
    }
}

impl IngestEventSource {
    pub fn new(api_key: ApiKey, site: Site) -> Self {
        Self { api_key, site }
    }
}

/// `CommonEvent` trait is used to represent the common attributes that all
/// event types must have in order to be valid.
pub trait CommonEvent {
    /// Retrieve the `ApiKey` for this event
    fn api_key(&self) -> &ApiKey;
    /// Retrieve the `Site` for this event
    fn site(&self) -> &Site;
    /// Retrieve the `Uuid` id for this event
    fn id(&self) -> Uuid;
    /// Retrieve the `OffsetDateTime` timestamp for this event
    fn ts(&self) -> &OffsetDateTime;
}

/// `VisitorEvent` represents a an event where an unrecognized user begins to
/// use the site. The `VisitorEvent` gives us a root against which all subsequent
/// events are based.
#[derive(Debug, Clone)]
pub struct VisitorEvent {
    /// `api_key` that ties this event to a particular client and site
    api_key: ApiKey,
    /// `site` is the site from which this event is coming. i.e. www.test.com
    site: Site,
    /// `id` is a `Uuid` that must be a UUIDv7 and must have an associated
    /// timestamp within a certain range of now in order to be considered valid
    /// for ingestion.
    id: Uuid,
    /// `ts` is the timestamp, represented as a
    /// `time::offset_date_time::OffsetDateTime` value. This is strictly derived
    /// from the `id` field above
    ts: OffsetDateTime,
}

impl CommonEvent for &VisitorEvent {
    fn api_key(&self) -> &ApiKey {
        &self.api_key
    }
    fn id(&self) -> Uuid {
        self.id
    }
    fn site(&self) -> &Site {
        &self.site
    }
    fn ts(&self) -> &OffsetDateTime {
        &self.ts
    }
}

impl VisitorEvent {
    /// `VisitorEvent` all field constructor
    pub fn try_new(api_key: ApiKey, site: Site, id: Uuid) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(IngestEventCore::try_new(api_key, site, id)?)
    }

    /// `VisitorEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    fn try_new_with_core_event(core: IngestEventCore) -> Result<Self, IngestEventError> {
        Ok(Self {
            api_key: core.api_key,
            id: core.id,
            site: core.site,
            ts: core.ts,
        })
    }
}

/// `SessionEvent` represents a new session start for an established `Visitor`
#[derive(Debug, Clone)]
pub struct SessionEvent {
    /// `api_key` that ties this event to a particular client and site
    api_key: ApiKey,
    /// `site` is the site from which this event is coming. i.e. www.test.com
    site: Site,
    /// `id` is a `Uuid` that must be a UUIDv7 and must have an associated
    /// timestamp within a certain range of now in order to be considered valid
    /// for ingestion.
    id: Uuid,
    /// `ts` is the timestamp, represented as a
    /// `time::offset_date_time::OffsetDateTime` value. This is strictly derived
    /// from the `id` field above
    ts: OffsetDateTime,
    /// `parent` identifies the `Visitor` which this session is associated with
    pub parent: Uuid,
    /// `user_agent` records the user agent/system on which the event originated
    pub user_agent: String,
}

impl CommonEvent for &SessionEvent {
    fn api_key(&self) -> &ApiKey {
        &self.api_key
    }
    fn id(&self) -> Uuid {
        self.id
    }
    fn site(&self) -> &Site {
        &self.site
    }
    fn ts(&self) -> &OffsetDateTime {
        &self.ts
    }
}

impl SessionEvent {
    /// `SessionEvent` all field constructor
    pub fn try_new(
        api_key: ApiKey,
        site: Site,
        id: Uuid,
        parent: Uuid,
        user_agent: String,
    ) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(
            IngestEventCore::try_new(api_key, site, id)?,
            parent,
            user_agent,
        )
    }

    /// `SessionEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    fn try_new_with_core_event(
        core: IngestEventCore,
        parent: Uuid,
        user_agent: String,
    ) -> Result<Self, IngestEventError> {
        Ok(Self {
            api_key: core.api_key,
            id: core.id,
            site: core.site,
            ts: core.ts,
            parent,
            user_agent,
        })
    }
}

/// `SectionEvent` represents an event for which the associated Visitor in a
/// given Session has been shown a Section. For a conventional web site this
/// would represent a page vend or page view. In a SPA this could mean that
/// a screen has been rendered. For a mobile app this could be a screen, a modal
/// or some other sort of interaction.
#[derive(Debug, Clone)]
pub struct SectionEvent {
    /// `api_key` that ties this event to a particular client and site
    api_key: ApiKey,
    /// `site` is the site from which this event is coming. i.e. www.test.com
    site: Site,
    /// `id` is a `Uuid` that must be a UUIDv7 and must have an associated
    /// timestamp within a certain range of now in order to be considered valid
    /// for ingestion.
    pub id: Uuid,
    /// `ts` is the timestamp, represented as a
    /// `time::offset_date_time::OffsetDateTime` value. This is strictly derived
    /// from the `id` field above
    ts: OffsetDateTime,
    /// `parent` identifies the `Session` which this section is associated with
    pub parent: Uuid,
    /// `location` specifies the full location string portion of the URI
    /// for the section, if it exists
    pub location: Option<String>,
    /// `title` identifies the title of the section, if it exists
    pub title: Option<String>,
}

impl CommonEvent for &SectionEvent {
    fn api_key(&self) -> &ApiKey {
        &self.api_key
    }
    fn id(&self) -> Uuid {
        self.id
    }
    fn site(&self) -> &Site {
        &self.site
    }
    fn ts(&self) -> &OffsetDateTime {
        &self.ts
    }
}

impl SectionEvent {
    /// `SectionEvent` all field constructor
    pub fn try_new(
        api_key: ApiKey,
        site: Site,
        id: Uuid,
        parent: Uuid,
        location: Option<String>,
        title: Option<String>,
    ) -> Result<Self, IngestEventError> {
        Self::try_new_with_core_event(
            IngestEventCore::try_new(api_key, site, id)?,
            parent,
            location,
            title,
        )
    }

    /// `SectionEvent` constructor with `IngestEventCore` already created for
    /// convenience or ergonomics
    fn try_new_with_core_event(
        core: IngestEventCore,
        parent: Uuid,
        location: Option<String>,
        title: Option<String>,
    ) -> Result<Self, IngestEventError> {
        Ok(Self {
            api_key: core.api_key,
            id: core.id,
            site: core.site,
            ts: core.ts,
            parent,
            location,
            title,
        })
    }
}

/// `ClickEvent` represents a user clicking or otherwise interacting with any
/// particular element in the interface of an associated Section.
#[derive(Debug, Clone)]
pub struct ClickEvent {
    /// `api_key` that ties this event to a particular client and site
    api_key: ApiKey,
    /// `site` is the site from which this event is coming. i.e. www.test.com
    site: Site,
    /// `id` is a `Uuid` that must be a UUIDv7 and must have an associated
    /// timestamp within a certain range of now in order to be considered valid
    /// for ingestion.
    id: Uuid,
    /// `ts` is the timestamp, represented as a
    /// `time::offset_date_time::OffsetDateTime` value. This is strictly derived
    /// from the `id` field above
    ts: OffsetDateTime,
    /// `parent` identifies the `Section` which this click is associated with
    pub parent: Uuid,
}

impl CommonEvent for &ClickEvent {
    fn api_key(&self) -> &ApiKey {
        &self.api_key
    }
    fn id(&self) -> Uuid {
        self.id
    }
    fn site(&self) -> &Site {
        &self.site
    }
    fn ts(&self) -> &OffsetDateTime {
        &self.ts
    }
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
    fn try_new_with_core_event(
        core: IngestEventCore,
        parent: Uuid,
    ) -> Result<Self, IngestEventError> {
        Ok(Self {
            api_key: core.api_key,
            id: core.id,
            site: core.site,
            ts: core.ts,
            parent,
        })
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
struct IngestEventCore {
    /// `api_key` that ties this event to a particular client and site
    api_key: ApiKey,
    /// `site` is the site from which this event is coming. i.e. www.test.com
    site: Site,
    /// `id` is a `Uuid` that must be a UUIDv7 and must have an associated
    /// timestamp within a certain range of now in order to be considered valid
    /// for ingestion.
    id: Uuid,
    /// `ts` is the timestamp, represented as a
    /// `time::offset_date_time::OffsetDateTime` value. This is strictly derived
    /// from the `id` field above
    ts: OffsetDateTime,
}

impl IngestEventCore {
    /// `IngestEventCore` constructor. Enforces domain rules with regard to
    /// `id` UUID type as well as the allowed range of times for events.
    pub fn try_new(api_key: ApiKey, site: Site, id: Uuid) -> Result<Self, IngestEventError> {
        if api_key.value().trim().is_empty() {
            return Err(IngestEventError::ApiKey);
        }
        if site.value().trim().is_empty() {
            return Err(IngestEventError::Site);
        }

        let ts = try_uuid_datetime(id)?;

        if is_ts_within_ingest_range(&ts) {
            Ok(Self {
                api_key,
                site,
                id,
                ts,
            })
        } else {
            Err(IngestEventError::TimestampOutOfRange)
        }
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
    fn test_core_event() {
        let uuid_now = Uuid::now_v7();
        let valid_core =
            IngestEventCore::try_new(ApiKey::new(API_KEY_STR), Site::new(SITE), uuid_now);
        if valid_core.is_err() {
            panic!("Expected a valid IngestEventCore to be created");
        }

        let invalid_api_key_err =
            IngestEventCore::try_new(ApiKey::new(" "), Site::new(SITE), uuid_now).unwrap_err();
        assert_eq!(
            invalid_api_key_err,
            IngestEventError::ApiKey,
            "Expected ApiKey error"
        );

        let invalid_site_err =
            IngestEventCore::try_new(ApiKey::new(API_KEY_STR), Site::new(" "), uuid_now)
                .unwrap_err();
        assert_eq!(
            invalid_site_err,
            IngestEventError::Site,
            "Expected Site error"
        );
    }

    #[test]
    fn test_try_new_events() {
        let uuid_now = Uuid::now_v7();
        let (ts_now, _) = uuid_now.get_timestamp().unwrap().to_unix();
        let test_core: Result<IngestEventCore, IngestEventError> =
            IngestEventCore::try_new(ApiKey::new(API_KEY_STR), Site::new(SITE), uuid_now);
        let Ok(_) = test_core else {
            panic!("Expected valid IngestEventCore");
        };

        let invalid_ingest_uuid_type: Result<IngestEventCore, IngestEventError> =
            IngestEventCore::try_new(
                ApiKey::new(API_KEY_STR),
                Site::new(SITE),
                Uuid::parse_str(UUID_V4_STR).unwrap(),
            );
        assert_eq!(
            invalid_ingest_uuid_type.unwrap_err(),
            IngestEventError::UuidVersion
        );

        let invalid_ingest_event_early: Result<IngestEventCore, IngestEventError> =
            IngestEventCore::try_new(
                ApiKey::new(API_KEY_STR),
                Site::new(SITE),
                Uuid::new_v7(Timestamp::from_unix_time(ts_now - 3601, 0, 0, 8)),
            );
        assert_eq!(
            invalid_ingest_event_early.unwrap_err(),
            IngestEventError::TimestampOutOfRange
        );

        let invalid_ingest_event_late: Result<IngestEventCore, IngestEventError> =
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
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0"
                .to_owned(),
        ) else {
            panic!("Expected valid SessionEvent");
        };

        let Ok(_) = SectionEvent::try_new(
            ApiKey::new(API_KEY_STR),
            Site::new(SITE),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Some("/path/to/section.ext?foo=bar&bar=foo#last".to_owned()),
            Some("Section Title".to_owned()),
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
