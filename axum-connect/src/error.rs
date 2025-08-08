// Already has documentation
//! Error and response types for Connect.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use connect_core::{Code, Error as ConnectCoreError};
use serde::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct ConnectError {
    code: Code,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    details: Vec<ErrorDetail>,
}

impl ConnectError {
    pub fn new<S: Into<String>>(code: Code, message: S) -> Self {
        Self {
            code,
            message: Some(message.into()),
            details: vec![],
        }
    }

    pub fn new_unimplemented() -> Self {
        Self {
            code: Code::Unimplemented,
            message: Some("The requested service has not been implemented.".to_string()),
            details: vec![],
        }
    }
}

impl IntoResponse for ConnectError {
    fn into_response(self) -> Response {
        let status_code = match self.code {
            Code::Canceled => StatusCode::REQUEST_TIMEOUT,
            Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Code::InvalidArgument => StatusCode::BAD_REQUEST,
            Code::DeadlineExceeded => StatusCode::REQUEST_TIMEOUT,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::AlreadyExists => StatusCode::CONFLICT,
            Code::PermissionDenied => StatusCode::FORBIDDEN,
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            Code::FailedPrecondition => StatusCode::BAD_REQUEST, // Connect spec says this should be 400
            Code::Aborted => StatusCode::CONFLICT,
            Code::OutOfRange => StatusCode::BAD_REQUEST,
            Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        };
        (status_code, Json(self)).into_response()
    }
}

impl From<ConnectCoreError> for ConnectError {
    fn from(err: ConnectCoreError) -> Self {
        Self {
            code: err.code,
            message: Some(err.message.to_string()),
            details: vec![], // TODO: map details
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: String, // Base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<serde_json::Value>,
}
