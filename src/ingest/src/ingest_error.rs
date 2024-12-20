use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represent potential error cases for an IngestEvent, either due to data
/// correctness issues or due to system availability problems.
#[derive(Clone, Error, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum IngestError {
    #[error("api key value missing from headers or invalid format")]
    ApiKey,
    #[error("Site missing from headers")]
    Site,
    #[error("Timestamp from UUID beyond acceptable range for new event")]
    TimestampOutOfRange,
    #[error("UUID version mismatch - must be UUIDv7")]
    UuidVersion,
    #[error("UUID timestamp conversion error")]
    UuidTimestampConversion,
}
