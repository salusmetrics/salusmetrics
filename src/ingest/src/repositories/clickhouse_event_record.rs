use clickhouse::Row;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    model::ingest_event::{
        ClickEvent, CoreEvent, IngestEvent, SectionEvent, SessionEvent, VisitorEvent,
    },
    repository::ingest_event_repository::IngestRepositoryError,
};

/// Type of analytics event - maps to ClickHouse Enum8 with identical values
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
/// SALUS_METRICS.EVENT table. Expected usages is to call
/// InsertIngestEvent::from_ingest_event on a IngestEvent that has been
/// received.
#[derive(Debug, Row, Deserialize, Serialize, Clone)]
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

impl TryFrom<&IngestEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    fn try_from(value: &IngestEvent) -> Result<Self, Self::Error> {
        match value {
            IngestEvent::Visitor(event) => event.try_into(),
            IngestEvent::Session(event) => event.try_into(),
            IngestEvent::Section(event) => event.try_into(),
            IngestEvent::Click(event) => event.try_into(),
        }
    }
}

impl TryFrom<&VisitorEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    fn try_from(event: &VisitorEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event.core);
        builder.try_build()
    }
}

impl TryFrom<&SessionEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    fn try_from(event: &SessionEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event.core);
        builder.add_parent(event.parent).try_build()
    }
}

impl TryFrom<&SectionEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    fn try_from(event: &SectionEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event.core);
        builder.add_parent(event.parent).try_build()
    }
}

impl TryFrom<&ClickEvent> for ClickhouseEventRecord {
    type Error = IngestRepositoryError;
    fn try_from(event: &ClickEvent) -> Result<Self, Self::Error> {
        let builder = ClickhouseEventRecordBuilder::from(&event.core);
        builder.add_parent(event.parent).try_build()
    }
}

struct ClickhouseEventRecordBuilder {
    api_key: String,
    site: String,
    id: Uuid,
    ts: OffsetDateTime,
    event_type: Option<ClickhouseEventRecordType>,
    attrs: Vec<(String, String)>,
}

impl<T> From<&CoreEvent<T>> for ClickhouseEventRecordBuilder {
    fn from(event: &CoreEvent<T>) -> Self {
        Self {
            api_key: event.api_key.0.to_owned(),
            site: event.site.0.to_owned(),
            id: event.id.to_owned(),
            ts: event.ts.to_owned(),
            event_type: None,
            attrs: Vec::new(),
        }
    }
}

impl ClickhouseEventRecordBuilder {
    fn event_type(mut self, event: &IngestEvent) -> Self {
        match event {
            IngestEvent::Visitor(_) => self.event_type = Some(ClickhouseEventRecordType::Visitor),
            IngestEvent::Session(_) => self.event_type = Some(ClickhouseEventRecordType::Session),
            IngestEvent::Section(_) => self.event_type = Some(ClickhouseEventRecordType::Section),
            IngestEvent::Click(_) => self.event_type = Some(ClickhouseEventRecordType::Click),
        }
        self
    }

    fn add_parent(self, parent: Uuid) -> Self {
        self.add_attr("parent".to_owned(), parent.to_string())
    }

    fn add_attr(mut self, key: String, value: String) -> Self {
        self.attrs.push((key, value));
        self
    }

    fn try_build(self) -> Result<ClickhouseEventRecord, IngestRepositoryError> {
        Ok(ClickhouseEventRecord {
            api_key: self.api_key,
            site: self.site,
            event_type: self.event_type.ok_or(IngestRepositoryError::Conversion)?,
            id: self.id,
            ts: self.ts,
            attrs: self.attrs,
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
        let uuid_now = Uuid::now_v7();
        let Ok(valid_visitor_event) = VisitorEvent::try_new(
            ApiKey("abc-124".to_owned()),
            Site("http://salusmetrics.com".to_owned()),
            uuid_now,
        ) else {
            panic!("Expected valid VisitorEvent to be created");
        };
        let Ok(_) = ClickhouseEventRecord::try_from(&IngestEvent::Visitor(valid_visitor_event))
        else {
            panic!("Expected valid ClickhouseEventRecord to be created from valid event");
        };
    }
}
