use std::collections::HashMap;

use axum::extract::FromRequestParts;
use http::HeaderMap;
use http::StatusCode;
use http::Uri;
use http::header;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::client_event_request::ClientEventRequestError;
use super::client_event_request::ClientEventRequestType;

/// `API_KEY_HTTP_HEADER` defines the name of the HTTP header that is examined
/// to determine a request's api_key
pub const API_KEY_HTTP_HEADER: &str = "api-key";

/// `ClientEventRequestBody` represents the interior fields an event request that an
/// external, untrusted client submits to the system.
///
/// `ClientEventRequestBody` is general across all types of events from new visitors down
/// to page actions. Handled by including `ClientEventRequestType` and generic attrs
/// HashMap to represent necessary data across types. Critically, the id must
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
    #[serde(alias = "t")]
    pub event_type: ClientEventRequestType,
    #[serde(alias = "i")]
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    #[serde(alias = "a")]
    pub attrs: Option<HashMap<String, String>>,
}

impl ClientEventRequestBody {
    /// `ClientEventRequestBody` constructor
    pub fn new(
        event_type: ClientEventRequestType,
        id: Uuid,
        attrs: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            event_type,
            id,
            attrs,
        }
    }
}

/// `ClientEventRequestHeaders` represents the information about a client-submitted event
/// which is not delivered in the body of the request, but rather from the
/// HTTP headers that arrive with that request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientEventRequestHeaders {
    pub api_key: String,
    pub site: String,
    pub user_agent: String,
}

impl ClientEventRequestHeaders {
    /// `ClientEventRequestHeaders` constructor. Takes an `ApiKey` and `Site` as arguments
    pub fn new(
        api_key: impl AsRef<str>,
        site: impl AsRef<str>,
        user_agent: impl AsRef<str>,
    ) -> Self {
        ClientEventRequestHeaders {
            api_key: api_key.as_ref().to_owned(),
            site: site.as_ref().to_owned(),
            user_agent: user_agent.as_ref().to_owned(),
        }
    }
}

/// `ClientEventRequestHeaders` must be able to be derived from incoming HTTP
/// headers alone.
impl TryFrom<&HeaderMap> for ClientEventRequestHeaders {
    type Error = ClientEventRequestError;

    fn try_from(value: &HeaderMap) -> Result<Self, Self::Error> {
        let origin = value
            .get(header::ORIGIN)
            .ok_or(ClientEventRequestError::InvalidRequestHeaders)?
            .to_str()
            .map_err(|_| ClientEventRequestError::InvalidRequestHeaders)?;
        let site = origin
            .parse::<Uri>()
            .map_err(|_| ClientEventRequestError::InvalidRequestHeaders)?
            .host()
            .ok_or(ClientEventRequestError::InvalidRequestHeaders)?
            .to_string();
        let api_key = value
            .get(API_KEY_HTTP_HEADER)
            .ok_or(ClientEventRequestError::ApiKey)?
            .to_str()
            .map_err(|_| ClientEventRequestError::ApiKey)?
            .to_string();
        let user_agent = value
            .get(header::USER_AGENT)
            .ok_or(ClientEventRequestError::InvalidRequestHeaders)?
            .to_str()
            .map_err(|_| ClientEventRequestError::InvalidRequestHeaders)?
            .to_string();
        Ok(ClientEventRequestHeaders {
            api_key,
            site,
            user_agent,
        })
    }
}

/// `ClientEventRequestHeaders` when handled by `FromRequestParts` allows the
/// handler methods to have arguments of type `ClientEventRequestHeaders`
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
    use http::{HeaderMap, header};

    use super::*;

    #[test]
    fn test_from_header_map() {
        // Positive test case
        let mut valid_headers = HeaderMap::new();
        valid_headers.insert(header::ORIGIN, "http://test.com/test/dir".parse().unwrap());
        valid_headers.insert(API_KEY_HTTP_HEADER, "1234-5678-90".parse().unwrap());
        valid_headers.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0"
                .parse()
                .unwrap(),
        );

        if ClientEventRequestHeaders::try_from(&valid_headers).is_err() {
            println!("headers: {valid_headers:?}");
            panic!("Expected valid ClientEventRequestHeaders for valid HeaderMap");
        }

        // Negative test cases
        let mut invalid_origin = HeaderMap::new();
        invalid_origin.insert(API_KEY_HTTP_HEADER, "1234-5678-90".parse().unwrap());
        invalid_origin.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            ClientEventRequestHeaders::try_from(&invalid_origin).unwrap_err(),
            ClientEventRequestError::InvalidRequestHeaders,
            "Should fail with no valid origin"
        );

        let mut invalid_api_key = HeaderMap::new();
        invalid_api_key.insert(header::ORIGIN, "http://test.com/test/dir".parse().unwrap());
        invalid_api_key.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            ClientEventRequestHeaders::try_from(&invalid_api_key).unwrap_err(),
            ClientEventRequestError::ApiKey,
            "Should fail with no valid api key"
        );

        let mut invalid_ua = HeaderMap::new();
        invalid_ua.insert(API_KEY_HTTP_HEADER, "1234-5678-90".parse().unwrap());
        invalid_ua.insert(header::ORIGIN, "http://test.com/test/dir".parse().unwrap());
        assert_eq!(
            ClientEventRequestHeaders::try_from(&invalid_ua).unwrap_err(),
            ClientEventRequestError::InvalidRequestHeaders,
            "Should fail with no valid user agent"
        );
    }
}
