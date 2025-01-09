use http::header;
use http::HeaderMap;
use http::Uri;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ingest_error::IngestError;

/// `ClientEvent` represents the unified event data that has been received
/// as a request from an external, untrusted client. This contains both
/// header-derived information and event-specific fields from the body
#[derive(Debug, Clone)]
pub struct ClientEvent {
    pub api_key: ApiKey,
    pub site: Site,
    pub event_type: ClientEventType,
    pub id: Uuid,
    pub attrs: Option<Vec<(String, String)>>,
}

impl ClientEvent {
    /// `ClientEvent` constructor from the request `EventHeaders` and specific
    /// event `ClientEventBody`
    pub fn new(headers: &EventHeaders, body: &ClientEventBody) -> Self {
        Self {
            api_key: headers.api_key(),
            site: headers.site(),
            event_type: body.event_type(),
            id: body.id(),
            attrs: body.attrs(),
        }
    }

    /// Getter to retrieve the associated `ApiKey` from this `ClientEvent`
    pub fn api_key(&self) -> ApiKey {
        self.api_key.to_owned()
    }

    /// Getter to retrieve the associated `Site` for this `ClientEvent`
    pub fn site(&self) -> Site {
        self.site.to_owned()
    }

    /// Getter for `ClientEventType` from this `ClientEvent`
    pub fn event_type(&self) -> ClientEventType {
        self.event_type.to_owned()
    }

    /// Getter for id of type `Uuid` from this `ClientEvent`
    pub fn id(&self) -> Uuid {
        self.id.to_owned()
    }

    /// Getter for the associated attrs from this `ClientEvent`
    pub fn attrs(&self) -> Option<Vec<(String, String)>> {
        self.attrs.to_owned()
    }
}

/// `ClientEventType` represents the type of analytics event submitted by
/// client. This enum must match up with the `event_record`'s
/// `EventRecordType` enum.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ClientEventType {
    Visitor,
    Session,
    Section,
    Click,
}

/// `ApiKey` is a newtype wrapper for handling api_key
/// The combination of `ApiKey` and `Site` determine whether an event is valid
/// for this instance and whether or not it will be stored.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKey(pub String);

/// `Site` is a newtype wrapper for handling site.
/// The combination of `ApiKey` and `Site` determine whether an event is valid
/// for this instance and whether or not it will be stored.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site(pub String);

/// `ClientEventBody` represents the interior fields an event request that an
/// external, untrusted client submits to the system.
///
/// `ClientEventBody` is general across all types of events from new visitors down
/// to page actions. Handled by including `ClientEventType` and generic attrs Vec
/// of tuples to represent necessary data across types. Critically, the id must
/// be a UUID v7 with the datetime portion of this id extracted to represent the
/// event time. If this time is not within a certain span of now, the event is
/// rejected.
///
/// The body and headers for event requests are kept separate so that a single
/// HTTP transaction can send multiple events simultaneously which share common
/// fields such as the api key and the origin that is contained in
/// `EventHeaders`, but then each event has distinct data that must be
/// represented as a `ClientEventBody`
///
/// The expectation for this event type is that some data is explicitly placed
/// in attrs by the client, but other data will be added by the server side.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientEventBody {
    event_type: ClientEventType,
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    attrs: Option<Vec<(String, String)>>,
}

impl ClientEventBody {
    /// `ClientEventBody` constructor
    pub fn new(
        event_type: ClientEventType,
        id: Uuid,
        attrs: Option<Vec<(String, String)>>,
    ) -> Self {
        Self {
            event_type,
            id,
            attrs,
        }
    }

    /// Getter for `ClientEventType` from this `ClientEventBody`
    pub fn event_type(&self) -> ClientEventType {
        self.event_type.to_owned()
    }

    /// Getter for id of type `Uuid` from this `ClientEventBody`
    pub fn id(&self) -> Uuid {
        self.id.to_owned()
    }

    /// Getter for the associated attrs from this `ClientEventBody`
    pub fn attrs(&self) -> Option<Vec<(String, String)>> {
        self.attrs.to_owned()
    }
}

/// `EventHeaders` represents the information about a client-submitted event
/// which is not delivered in the body of the request, but rather from the
/// HTTP headers that arrive with that request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventHeaders {
    api_key: ApiKey,
    site: Site,
}

impl EventHeaders {
    /// `EventHeaders` constructor. Takes an `ApiKey` and `Site` as arguments
    pub fn new(api_key: ApiKey, site: Site) -> Self {
        EventHeaders { api_key, site }
    }

    /// Getter to retrieve the associated `ApiKey` from this `EventHeaders`
    pub fn api_key(&self) -> ApiKey {
        self.api_key.to_owned()
    }

    /// Getter to retrieve the associated `Site` for this `EventHeaders`
    pub fn site(&self) -> Site {
        self.site.to_owned()
    }
}

impl TryFrom<&HeaderMap> for EventHeaders {
    type Error = IngestError;

    fn try_from(value: &HeaderMap) -> Result<Self, Self::Error> {
        let referer = value
            .get(header::REFERER)
            .ok_or(IngestError::Site)?
            .to_str()
            .map_err(|_| IngestError::Site)?;
        let host = referer
            .parse::<Uri>()
            .map_err(|_| IngestError::Site)?
            .host()
            .ok_or(IngestError::Site)?
            .to_string();
        let site = Site(host);
        let api_key = ApiKey(
            value
                .get("api-key")
                .ok_or(IngestError::ApiKey)?
                .to_str()
                .map_err(|_| IngestError::ApiKey)?
                .to_string(),
        );
        Ok(EventHeaders { api_key, site })
    }
}

#[cfg(test)]
mod tests {
    use http::{header, HeaderMap};

    use crate::ingest_error::IngestError;

    use super::EventHeaders;

    #[test]
    fn test_from_header_map() {
        // Positive test case
        let mut valid_headers = HeaderMap::new();
        valid_headers.insert(header::REFERER, "http://test.com/test/dir".parse().unwrap());
        valid_headers.insert("api-key", "1234-5678-90".parse().unwrap());

        if EventHeaders::try_from(&valid_headers).is_err() {
            println!("headers: {:?}", valid_headers);
            panic!("Expected valid EventHeaders for valid HeaderMap");
        }

        // Negative test cases
        let mut invalid_referer = HeaderMap::new();
        invalid_referer.insert("api-key", "1234-5678-90".parse().unwrap());
        assert_eq!(
            EventHeaders::try_from(&invalid_referer).unwrap_err(),
            IngestError::Site,
            "Should fail with no valid refere"
        );

        let mut invalid_api_key = HeaderMap::new();
        invalid_api_key.insert(header::REFERER, "http://test.com/test/dir".parse().unwrap());
        assert_eq!(
            EventHeaders::try_from(&invalid_api_key).unwrap_err(),
            IngestError::ApiKey,
            "Should fail with no valid refere"
        );
    }
}
