use axum::extract::FromRequestParts;
use http::header;
use http::HeaderMap;
use http::StatusCode;
use http::Uri;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::ingest_event_error::IngestEventError;

/// `ClientEventRequestType` represents the type of analytics event submitted by
/// client. This enum must match up with the `event_record`'s
/// `EventRecordType` enum.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ClientEventRequestType {
    Visitor,
    Session,
    Section,
    Click,
}

/// `ClientEventRequestBody` represents the interior fields an event request that an
/// external, untrusted client submits to the system.
///
/// `ClientEventRequestBody` is general across all types of events from new visitors down
/// to page actions. Handled by including `ClientEventRequestType` and generic attrs Vec
/// of tuples to represent necessary data across types. Critically, the id must
/// be a UUID v7 with the datetime portion of this id extracted to represent the
/// event time. If this time is not within a certain span of now, the event is
/// rejected.
///
/// The body and headers for event requests are kept separate so that a single
/// HTTP transaction can send multiple events simultaneously which share common
/// fields such as the api key and the origin that is contained in
/// `ClientEventRequestHeaders`, but then each event has distinct data that must be
/// represented as a `ClientEventRequestBody`
///
/// The expectation for this event type is that some data is explicitly placed
/// in attrs by the client, but other data will be added by the server side.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientEventRequestBody {
    event_type: ClientEventRequestType,
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    attrs: Option<Vec<(String, String)>>,
}

impl ClientEventRequestBody {
    /// `ClientEventRequestBody` constructor
    pub fn new(
        event_type: ClientEventRequestType,
        id: Uuid,
        attrs: Option<Vec<(String, String)>>,
    ) -> Self {
        Self {
            event_type,
            id,
            attrs,
        }
    }

    /// Getter for `ClientEventRequestType` from this `ClientEventRequestBody`
    pub fn event_type(&self) -> &ClientEventRequestType {
        &self.event_type
    }

    /// Getter for id of type `Uuid` from this `ClientEventRequestBody`
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Getter for the associated attrs from this `ClientEventRequestBody`
    pub fn attrs(&self) -> &Option<Vec<(String, String)>> {
        &self.attrs
    }
}

/// `ClientEventRequestHeaders` represents the information about a client-submitted event
/// which is not delivered in the body of the request, but rather from the
/// HTTP headers that arrive with that request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientEventRequestHeaders {
    api_key: String,
    site: String,
}

impl ClientEventRequestHeaders {
    /// `ClientEventRequestHeaders` constructor. Takes an `ApiKey` and `Site` as arguments
    pub fn new(api_key: impl AsRef<str>, site: impl AsRef<str>) -> Self {
        ClientEventRequestHeaders {
            api_key: api_key.as_ref().to_owned(),
            site: site.as_ref().to_owned(),
        }
    }

    /// Getter to retrieve the associated `ApiKey` from this `ClientEventRequestHeaders`
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Getter to retrieve the associated `Site` for this `ClientEventRequestHeaders`
    pub fn site(&self) -> &str {
        &self.site
    }
}

impl TryFrom<&HeaderMap> for ClientEventRequestHeaders {
    type Error = IngestEventError;

    fn try_from(value: &HeaderMap) -> Result<Self, Self::Error> {
        let referer = value
            .get(header::REFERER)
            .ok_or(IngestEventError::Site)?
            .to_str()
            .map_err(|_| IngestEventError::Site)?;
        let site = referer
            .parse::<Uri>()
            .map_err(|_| IngestEventError::Site)?
            .host()
            .ok_or(IngestEventError::Site)?
            .to_string();
        let api_key = value
            .get("api-key")
            .ok_or(IngestEventError::ApiKey)?
            .to_str()
            .map_err(|_| IngestEventError::ApiKey)?
            .to_string();
        Ok(ClientEventRequestHeaders { api_key, site })
    }
}

impl<S> FromRequestParts<S> for ClientEventRequestHeaders
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        Self::try_from(&parts.headers).map_err(|_| (StatusCode::BAD_REQUEST, "Bad Request"))
    }
}

#[cfg(test)]
mod tests {
    use http::{header, HeaderMap};

    use super::*;
    use crate::ingest_error::IngestEventError;

    #[test]
    fn test_from_header_map() {
        // Positive test case
        let mut valid_headers = HeaderMap::new();
        valid_headers.insert(header::REFERER, "http://test.com/test/dir".parse().unwrap());
        valid_headers.insert("api-key", "1234-5678-90".parse().unwrap());

        if ClientEventRequestHeaders::try_from(&valid_headers).is_err() {
            println!("headers: {:?}", valid_headers);
            panic!("Expected valid ClientEventRequestHeaders for valid HeaderMap");
        }

        // Negative test cases
        let mut invalid_referer = HeaderMap::new();
        invalid_referer.insert("api-key", "1234-5678-90".parse().unwrap());
        assert_eq!(
            ClientEventRequestHeaders::try_from(&invalid_referer).unwrap_err(),
            IngestEventError::Site,
            "Should fail with no valid refere"
        );

        let mut invalid_api_key = HeaderMap::new();
        invalid_api_key.insert(header::REFERER, "http://test.com/test/dir".parse().unwrap());
        assert_eq!(
            ClientEventRequestHeaders::try_from(&invalid_api_key).unwrap_err(),
            IngestEventError::ApiKey,
            "Should fail with no valid refere"
        );
    }
}
