use clickhouse::Row;
use serde::{Deserialize, Serialize};

use crate::domain::model::ingest_event::{ApiKey, IngestEventSource, Site};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Row, Deserialize, Serialize)]
pub struct ClickhouseSourceRecord {
    api_key: String,
    site: String,
}

impl ClickhouseSourceRecord {
    pub fn new(api_key: impl AsRef<str>, site: impl AsRef<str>) -> Self {
        Self {
            api_key: api_key.as_ref().to_string(),
            site: site.as_ref().to_string(),
        }
    }
}

impl From<&ClickhouseSourceRecord> for IngestEventSource {
    fn from(value: &ClickhouseSourceRecord) -> Self {
        Self::new(ApiKey::new(&value.api_key), Site::new(&value.site))
    }
}
