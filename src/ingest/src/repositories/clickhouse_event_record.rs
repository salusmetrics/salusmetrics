use std::collections::HashSet;

use clickhouse::Row;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::{
    model::ingest_event::{
        ClickEvent, CommonEvent, IngestEvent, SectionEvent, SessionEvent, VisitorEvent,
    },
    repository::ingest_event_repository::IngestRepositoryError,
};

/// `ClickhouseEventRecordType` represents the type of analytics event - maps
/// to ClickHouse Enum8 with identical values.
/// See definition of table `SALUS_METRICS.EVENT` and field `event_type`
#[derive(Debug, Deserialize_repr, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Clone)]
#[repr(u8)]
pub enum ClickhouseEventRecordType {
    Visitor = 1,
    Session = 2,
    Section = 3,
    Click = 4,
}

impl From<&IngestEvent> for ClickhouseEventRecordType {
    #[instrument]
    fn from(value: &IngestEvent) -> Self {
        match value {
            IngestEvent::Visitor(_) => Self::Visitor,
            IngestEvent::Session(_) => Self::Session,
            IngestEvent::Section(_) => Self::Section,
            IngestEvent::Click(_) => Self::Click,
        }
    }
}

/// Represents the data that will actually be inserted into ClickHouse in the
/// SALUS_METRICS.EVENT table.
/// Note that this representation differs from the `IngestEvent` domain struct
/// in numerous ways, but also strongly differs from the HTTP representation
/// that is specified in `ClientEventRequest`. The biggest reason for this is
/// because all events persisted to ClickHouse start off as records in a
/// the `EVENT` table and thus have to store all non-common attributes in
/// a (String, String) tuple.
#[derive(Debug, Row, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ClickhouseEventRecord {
    api_key: String,
    site: String,
    event_type: ClickhouseEventRecordType,
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    ts: OffsetDateTime,
    attrs: Vec<(String, String)>,
}

/// `ClickhouseEventRecord` translates from the core `IngestEvent` domain
/// model into something that can be persisted to the Clickhouse DB
impl TryFrom<&IngestEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    #[instrument]
    fn try_from(value: &IngestEvent) -> Result<Self, Self::Error> {
        match value {
            IngestEvent::Visitor(event) => event.try_into(),
            IngestEvent::Session(event) => event.try_into(),
            IngestEvent::Section(event) => event.try_into(),
            IngestEvent::Click(event) => event.try_into(),
        }
    }
}

/// `ClickhouseEventRecord` derived from each `IngestEvent` type's discriminant
/// `Visitor` discriminant
impl TryFrom<&VisitorEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    #[instrument]
    fn try_from(event: &VisitorEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event);
        builder
            .event_type(ClickhouseEventRecordType::Visitor)
            .try_build()
    }
}

/// `ClickhouseEventRecord` derived from each `IngestEvent` type's discriminant
/// `Session` discriminant
impl TryFrom<&SessionEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    #[instrument]
    fn try_from(event: &SessionEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event);
        builder
            .event_type(ClickhouseEventRecordType::Session)
            .parent(event.parent())
            .try_build()
    }
}

/// `ClickhouseEventRecord` derived from each `IngestEvent` type's discriminant
/// `Section` discriminant
impl TryFrom<&SectionEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    #[instrument]
    fn try_from(event: &SectionEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event);
        builder
            .event_type(ClickhouseEventRecordType::Session)
            .parent(event.parent())
            .try_build()
    }
}

/// `ClickhouseEventRecord` derived from each `IngestEvent` type's discriminant
/// `Click` discriminant
impl TryFrom<&ClickEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    #[instrument]
    fn try_from(event: &ClickEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event);
        builder
            .event_type(ClickhouseEventRecordType::Click)
            .parent(event.parent())
            .try_build()
    }
}

/// `ClickhouseEventRecordBuilder` is an internal struct used to build up a
/// `ClickhouseEventRecord` in an ergonomic way. Part of this relies on the
/// `CommonEvent` trait that is provided in the domain to represent the fields
/// that all event types must have in order to be saved.
struct ClickhouseEventRecordBuilder {
    api_key: String,
    site: String,
    id: Uuid,
    ts: OffsetDateTime,
    event_type: Option<ClickhouseEventRecordType>,
    attrs: HashSet<(String, String)>,
}

/// `ClickhouseEventRecordBuilder` ergonomic conversion from the `CommonEvent`
/// trait. This takes care of the core data fields of `api_key`, `site`, `id`
/// and `ts`
impl<T> From<&T> for ClickhouseEventRecordBuilder
where
    T: CommonEvent,
{
    fn from(event: &T) -> Self {
        Self {
            api_key: event.api_key().value().to_owned(),
            site: event.site().value().to_owned(),
            id: event.id().to_owned(),
            ts: event.ts().to_owned(),
            event_type: None,
            attrs: HashSet::new(),
        }
    }
}

impl ClickhouseEventRecordBuilder {
    /// Set the `event_type` `ClickhouseEventRecordType` for the eventual
    /// `ClickhouseEventRecord`
    fn event_type(mut self, event_type: ClickhouseEventRecordType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Set the `parent` field on the eventual `ClickhouseEventRecord`
    fn parent(self, parent: Uuid) -> Self {
        self.add_attr("parent".to_owned(), parent.to_string())
    }

    /// Helper method to add an arbitrarily named attribute to the eventual
    /// `ClickhouseEventRecord`
    fn add_attr(mut self, key: String, value: String) -> Self {
        self.attrs.insert((key, value));
        self
    }

    /// Attempt to actually create the `ClickhouseEventRecord` from this
    /// `ClickhouseEventRecordBuilder`
    fn try_build(self) -> Result<ClickhouseEventRecord, IngestRepositoryError> {
        // let attrs: Vec<(String, String)> = HashSet::with_capacity(self.attrs.len());
        // for pair in self.attrs.iter() {
        //     attrs.push(pair);
        // }
        Ok(ClickhouseEventRecord {
            api_key: self.api_key,
            site: self.site,
            event_type: self.event_type.ok_or(IngestRepositoryError::Conversion)?,
            id: self.id,
            ts: self.ts,
            attrs: self.attrs.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::model::ingest_event::{ApiKey, Site};

    use super::*;

    /// This is very important in order to keep the mapping in ClickHouse in
    /// line with this library
    #[test]
    fn test_event_record_type_discriminant() {
        let visitor_discriminant = ClickhouseEventRecordType::Visitor as u32;
        assert_eq!(
            visitor_discriminant, 1,
            "ClickhouseEventRecordType::Visitor discriminant does not match expected value"
        );

        let session_discriminant = ClickhouseEventRecordType::Session as u32;
        assert_eq!(
            session_discriminant, 2,
            "ClickhouseEventRecordType::Session discriminant does not match expected value"
        );

        let section_discriminant = ClickhouseEventRecordType::Section as u32;
        assert_eq!(
            section_discriminant, 3,
            "ClickhouseEventRecordType::Section discriminant does not match expected value"
        );

        let click_discriminant = ClickhouseEventRecordType::Click as u32;
        assert_eq!(
            click_discriminant, 4,
            "ClickhouseEventRecordType::Click discriminant does not match expected value"
        );
    }

    #[test]
    fn test_try_from_ingest_event() {
        let uuid_visitor = Uuid::now_v7();
        let Ok(valid_visitor_event) = VisitorEvent::try_new(
            ApiKey::new("abc-124"),
            Site::new("http://salusmetrics.com"),
            uuid_visitor,
        ) else {
            panic!("Expected valid VisitorEvent to be created");
        };
        let Ok(_) = ClickhouseEventRecord::try_from(&IngestEvent::Visitor(valid_visitor_event))
        else {
            panic!("Expected valid Visitor ClickhouseEventRecord to be created from valid event");
        };

        let uuid_session = Uuid::now_v7();
        let Ok(valid_session_event) = SessionEvent::try_new(
            ApiKey::new("abc-124"),
            Site::new("http://salusmetrics.com"),
            uuid_session,
            uuid_visitor,
        ) else {
            panic!("Expected valid SessionEvent to be created");
        };
        let Ok(_) = ClickhouseEventRecord::try_from(&IngestEvent::Session(valid_session_event))
        else {
            panic!("Expected valid Session ClickhouseEventRecord to be created from valid event");
        };

        let uuid_section = Uuid::now_v7();
        let Ok(valid_section_event) = SectionEvent::try_new(
            ApiKey::new("abc-124"),
            Site::new("http://salusmetrics.com"),
            uuid_section,
            uuid_session,
        ) else {
            panic!("Expected valid SectionEvent to be created");
        };
        let Ok(_) = ClickhouseEventRecord::try_from(&IngestEvent::Section(valid_section_event))
        else {
            panic!("Expected valid Section ClickhouseEventRecord to be created from valid event");
        };

        let uuid_click = Uuid::now_v7();
        let Ok(valid_click_event) = ClickEvent::try_new(
            ApiKey::new("abc-124"),
            Site::new("http://salusmetrics.com"),
            uuid_click,
            uuid_section,
        ) else {
            panic!("Expected valid ClickEvent to be created");
        };
        let Ok(_) = ClickhouseEventRecord::try_from(&IngestEvent::Click(valid_click_event)) else {
            panic!("Expected valid Click ClickhouseEventRecord to be created from valid event");
        };
    }
}
