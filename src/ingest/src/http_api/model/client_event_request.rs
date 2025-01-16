use axum::response::IntoResponse;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::model::ingest_event::ApiKey;
use crate::domain::model::ingest_event::ClickEvent;
use crate::domain::model::ingest_event::IngestEvent;
use crate::domain::model::ingest_event::IngestEventError;
use crate::domain::model::ingest_event::SectionEvent;
use crate::domain::model::ingest_event::SessionEvent;
use crate::domain::model::ingest_event::Site;
use crate::domain::model::ingest_event::VisitorEvent;
use crate::domain::service::ingest_event_service::IngestServiceError;

use super::client_event_request_components::ClientEventRequestBody;
use super::client_event_request_components::ClientEventRequestHeaders;

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

/// `ClientEventRequestError` encapsulates the error types that can occur
/// at the HTTP tier. This is primarily through wrapping errors that can arise
/// at lower layers.
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ClientEventRequestError {
    #[error("API KEY missing from request header")]
    ApiKey,
    #[error("Error converting client request into ingest event")]
    IngestEvent(#[from] IngestEventError),
    #[error("Error returned from IngestEventService")]
    IngestService(#[from] IngestServiceError),
    #[error("Invalid request body")]
    InvalidRequestBody,
    #[error("Invalid request headers")]
    InvalidRequestHeaders,
    #[error("Somehow ended up trying to create event of one type with input for another - this should never happen")]
    TypeMismatch,
}

/// `ClientEventRequestError` needs to implement `IntoResponse` in order to
/// provide an ergonomic method signature wherein all HTTP handlers can return
/// `Result<ClientEventActionSummary, ClientEventRequestError>` and have both
/// portions of the `Result` properly implement `IntoResponse`
/// Note that this response is meant to not reveal any internal error
/// information to the client for the sake of security. Instead, all responses
/// are merely HTTP response codes with no accompanying body.
impl IntoResponse for ClientEventRequestError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ClientEventRequestError::ApiKey => StatusCode::BAD_REQUEST.into_response(),
            ClientEventRequestError::InvalidRequestBody => StatusCode::BAD_REQUEST.into_response(),
            ClientEventRequestError::InvalidRequestHeaders => {
                StatusCode::BAD_REQUEST.into_response()
            }
            ClientEventRequestError::IngestEvent(e) => {
                tracing::error!("{}", e);
                StatusCode::BAD_REQUEST.into_response()
            }
            ClientEventRequestError::IngestService(e) => {
                tracing::error!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ClientEventRequestError::TypeMismatch => {
                tracing::error!("Encounterd TypeMismatch, which shoule never happen");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

/// `ClientEventRequest` represents an external metrics event from an untrusted
/// HTTP source. Crucially, data to build up this request comes from both the
/// headers of the HTTP request as well as from the body of the request. Each
/// event in a full HTTP request is individually constructed.
pub struct ClientEventRequest {
    pub headers: ClientEventRequestHeaders,
    pub body: ClientEventRequestBody,
}

impl ClientEventRequest {
    /// `attr` method provides an ergonomic way to access possible attributes
    /// that were specified in the body of the request for a given event
    pub fn attr(&self, key: &str) -> Option<&String> {
        match &self.body.attrs {
            None => None,
            Some(attrs) => attrs.get(key),
        }
    }
}

/// `ClientEventRequest` needs to be able to be translated into the domain
/// object of `IngestEvent`. This call can fail because domain rules are
/// applied at construction time. This is a two-part step in order to create
/// both the outer `IngestEvent` enum variant as well as the discriminant
/// type for each.
impl TryFrom<&ClientEventRequest> for IngestEvent {
    type Error = ClientEventRequestError;

    fn try_from(value: &ClientEventRequest) -> Result<Self, Self::Error> {
        match &value.body.event_type {
            ClientEventRequestType::Visitor => Ok(IngestEvent::Visitor(value.try_into()?)),
            ClientEventRequestType::Session => Ok(IngestEvent::Session(value.try_into()?)),
            ClientEventRequestType::Section => Ok(IngestEvent::Section(value.try_into()?)),
            ClientEventRequestType::Click => Ok(IngestEvent::Click(value.try_into()?)),
        }
    }
}

/// `ClientEventRequest` to the discriminant for `IngestEvent::Visitor`
impl TryFrom<&ClientEventRequest> for VisitorEvent {
    type Error = ClientEventRequestError;
    fn try_from(value: &ClientEventRequest) -> Result<Self, Self::Error> {
        assert!(
            value.body.event_type == ClientEventRequestType::Visitor,
            "Attempted to build Visitor event from other type"
        );

        VisitorEvent::try_new(
            ApiKey::new(&value.headers.api_key),
            Site::new(&value.headers.site),
            value.body.id,
        )
        .map_err(|e| e.into())
    }
}

/// `ClientEventRequest` to the discriminant for `IngestEvent::Session`
impl TryFrom<&ClientEventRequest> for SessionEvent {
    type Error = ClientEventRequestError;
    fn try_from(value: &ClientEventRequest) -> Result<Self, Self::Error> {
        assert!(
            value.body.event_type == ClientEventRequestType::Session,
            "Attempted to build Session event from other type"
        );

        let parent = value
            .attr("parent")
            .ok_or(ClientEventRequestError::InvalidRequestBody)?;
        let parent_uuid =
            Uuid::parse_str(parent).map_err(|_| ClientEventRequestError::InvalidRequestBody)?;
        SessionEvent::try_new(
            ApiKey::new(&value.headers.api_key),
            Site::new(&value.headers.site),
            value.body.id,
            parent_uuid,
        )
        .map_err(|e| e.into())
    }
}

/// `ClientEventRequest` to the discriminant for `IngestEvent::Section`
impl TryFrom<&ClientEventRequest> for SectionEvent {
    type Error = ClientEventRequestError;
    fn try_from(value: &ClientEventRequest) -> Result<Self, Self::Error> {
        assert!(
            value.body.event_type == ClientEventRequestType::Section,
            "Attempted to build Section event from other type"
        );

        let parent = value
            .attr("parent")
            .ok_or(ClientEventRequestError::InvalidRequestBody)?;
        let parent_uuid =
            Uuid::parse_str(parent).map_err(|_| ClientEventRequestError::InvalidRequestBody)?;
        SectionEvent::try_new(
            ApiKey::new(&value.headers.api_key),
            Site::new(&value.headers.site),
            value.body.id,
            parent_uuid,
        )
        .map_err(|e| e.into())
    }
}

/// `ClientEventRequest` to the discriminant for `IngestEvent::Click`
impl TryFrom<&ClientEventRequest> for ClickEvent {
    type Error = ClientEventRequestError;
    fn try_from(value: &ClientEventRequest) -> Result<Self, Self::Error> {
        assert!(
            value.body.event_type == ClientEventRequestType::Click,
            "Attempted to build Click event from other type"
        );

        let parent = value
            .attr("parent")
            .ok_or(ClientEventRequestError::InvalidRequestBody)?;
        let parent_uuid =
            Uuid::parse_str(parent).map_err(|_| ClientEventRequestError::InvalidRequestBody)?;
        ClickEvent::try_new(
            ApiKey::new(&value.headers.api_key),
            Site::new(&value.headers.site),
            value.body.id,
            parent_uuid,
        )
        .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;
    use crate::domain::model::ingest_event::CommonEvent;

    pub const API_KEY: &str = "abc_123";
    pub const SITE: &str = "test.com";
    #[test]
    fn test_try_from_client_request() {
        let uuid_now = Uuid::now_v7();
        let parent_id = Uuid::now_v7();

        // Visitor
        let valid_visitor_request = ClientEventRequest {
            body: ClientEventRequestBody {
                id: uuid_now,
                event_type: ClientEventRequestType::Visitor,
                attrs: None,
            },
            headers: ClientEventRequestHeaders {
                api_key: API_KEY.to_owned(),
                site: SITE.to_owned(),
            },
        };
        let visitor_ingest_event: IngestEvent = (&valid_visitor_request).try_into().unwrap();
        match visitor_ingest_event {
            IngestEvent::Visitor(ref visitor_event) => {
                assert_eq!(visitor_event.api_key().value(), API_KEY);
                assert_eq!(visitor_event.site().value(), SITE);
                assert_eq!(visitor_event.id(), uuid_now);
            }
            _ => panic!("Expected valid visitor event to be generated"),
        }

        // Session
        let session_attrs: HashMap<String, String> =
            HashMap::from([("parent".to_owned(), parent_id.to_string())]);
        let valid_session_request = ClientEventRequest {
            body: ClientEventRequestBody {
                id: uuid_now,
                event_type: ClientEventRequestType::Session,
                attrs: Some(session_attrs),
            },
            headers: ClientEventRequestHeaders {
                api_key: API_KEY.to_owned(),
                site: SITE.to_owned(),
            },
        };
        let session_ingest_event: IngestEvent = (&valid_session_request).try_into().unwrap();
        match session_ingest_event {
            IngestEvent::Session(ref session_event) => {
                assert_eq!(session_event.api_key().value(), API_KEY);
                assert_eq!(session_event.site().value(), SITE);
                assert_eq!(session_event.id(), uuid_now);
                assert_eq!(session_event.parent(), parent_id);
            }
            _ => panic!("Expected valid session event to be generated"),
        }

        // Section
        let section_attrs: HashMap<String, String> =
            HashMap::from([("parent".to_owned(), parent_id.to_string())]);
        let valid_section_request = ClientEventRequest {
            body: ClientEventRequestBody {
                id: uuid_now,
                event_type: ClientEventRequestType::Section,
                attrs: Some(section_attrs),
            },
            headers: ClientEventRequestHeaders {
                api_key: API_KEY.to_owned(),
                site: SITE.to_owned(),
            },
        };
        let section_ingest_event: IngestEvent = (&valid_section_request).try_into().unwrap();
        match section_ingest_event {
            IngestEvent::Section(ref section_event) => {
                assert_eq!(section_event.api_key().value(), API_KEY);
                assert_eq!(section_event.site().value(), SITE);
                assert_eq!(section_event.id(), uuid_now);
                assert_eq!(section_event.parent(), parent_id);
            }
            _ => panic!("Expected valid section event to be generated"),
        }

        // Click
        let click_attrs: HashMap<String, String> =
            HashMap::from([("parent".to_owned(), parent_id.to_string())]);
        let valid_click_request = ClientEventRequest {
            body: ClientEventRequestBody {
                id: uuid_now,
                event_type: ClientEventRequestType::Click,
                attrs: Some(click_attrs),
            },
            headers: ClientEventRequestHeaders {
                api_key: API_KEY.to_owned(),
                site: SITE.to_owned(),
            },
        };
        let click_ingest_event: IngestEvent = (&valid_click_request).try_into().unwrap();
        match click_ingest_event {
            IngestEvent::Click(ref click_event) => {
                assert_eq!(click_event.api_key().value(), API_KEY);
                assert_eq!(click_event.site().value(), SITE);
                assert_eq!(click_event.id(), uuid_now);
                assert_eq!(click_event.parent(), parent_id);
            }
            _ => panic!("Expected valid section event to be generated"),
        }
    }
}
