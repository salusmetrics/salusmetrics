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

#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ClientEventRequestError {
    #[error("API KEY missing from request header")]
    ApiKey,
    #[error("Error converting client request into ingest event")]
    IngestEvent(#[from] IngestEventError),
    #[error("Error returned from IngestEventService")]
    IngestService(#[from] IngestServiceError),
    #[error("Invalid request headers")]
    InvalidRequestBody,
    #[error("Invalid request body")]
    InvalidRequestHeaders,
    #[error("Somehow ended up trying to create event of one type with input for another - this should never happen")]
    TypeMismatch,
}

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

pub struct ClientEventRequest {
    pub headers: ClientEventRequestHeaders,
    pub body: ClientEventRequestBody,
}

impl ClientEventRequest {
    pub fn attr(&self, key: &str) -> Option<&String> {
        match &self.body.attrs {
            None => None,
            Some(attrs) => attrs.get(key),
        }
    }
}

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

impl TryFrom<&ClientEventRequest> for VisitorEvent {
    type Error = ClientEventRequestError;
    fn try_from(value: &ClientEventRequest) -> Result<Self, Self::Error> {
        assert!(
            value.body.event_type == ClientEventRequestType::Visitor,
            "Attempted to build Visitor event from other type"
        );

        VisitorEvent::try_new(
            ApiKey(value.headers.api_key.to_owned()),
            Site(value.headers.site.to_owned()),
            value.body.id,
        )
        .map_err(|e| e.into())
    }
}

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
            ApiKey(value.headers.api_key.to_owned()),
            Site(value.headers.site.to_owned()),
            value.body.id,
            parent_uuid,
        )
        .map_err(|e| e.into())
    }
}

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
            ApiKey(value.headers.api_key.to_owned()),
            Site(value.headers.site.to_owned()),
            value.body.id,
            parent_uuid,
        )
        .map_err(|e| e.into())
    }
}

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
            ApiKey(value.headers.api_key.to_owned()),
            Site(value.headers.site.to_owned()),
            value.body.id,
            parent_uuid,
        )
        .map_err(|e| e.into())
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_try_from_client_request() {

//     }
// }
