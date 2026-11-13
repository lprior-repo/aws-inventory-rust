//! Typed JSON failures: every refusal has a stable `code` and a message.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cmdb_view::ViewError;
use serde::Serialize;

/// A request-level failure rendered as `{"error":{"code","message"}}`.
#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    /// Unparsable query string or path encoding.
    pub fn malformed_query() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "malformed_query",
            "query or path parameters could not be parsed",
        )
    }

    /// A path segment contained invalid percent encoding.
    pub fn malformed_id() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "malformed_id",
            "resource id is not a valid path segment",
        )
    }

    /// Query string exceeded [`crate::MAX_QUERY_BYTES`].
    pub fn query_too_long(limit: usize) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "query_too_long",
            format!("query string exceeds the {limit}-byte limit"),
        )
    }

    /// Declared request body exceeded [`crate::MAX_BODY_BYTES`].
    pub fn payload_too_large(limit: usize) -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            format!("request body exceeds the {limit}-byte limit"),
        )
    }

    /// Unknown endpoint or unknown resource identity.
    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message)
    }

    /// Unknown resource or graph root identity.
    pub fn resource_not_found(id: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "resource_not_found",
            format!("resource {id:?} is not in this snapshot"),
        )
    }

    /// Route exists but the method is not readable.
    pub fn method_not_allowed() -> Self {
        Self::new(
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "only GET requests are supported",
        )
    }

    /// Map a view-layer bound violation to the matching HTTP status.
    pub fn from_view(error: ViewError) -> Self {
        let (status, code) = match &error {
            ViewError::InvalidPageLimit => (StatusCode::BAD_REQUEST, "invalid_page_limit"),
            ViewError::InvalidCursor => (StatusCode::BAD_REQUEST, "invalid_cursor"),
            ViewError::InvalidGraphDepth => (StatusCode::BAD_REQUEST, "invalid_graph_depth"),
            ViewError::InvalidGraphLimit => (StatusCode::BAD_REQUEST, "invalid_graph_limit"),
            ViewError::InvalidLodLimit => (StatusCode::BAD_REQUEST, "invalid_lod_limit"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl From<ViewError> for AppError {
    fn from(error: ViewError) -> Self {
        Self::from_view(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let payload = ErrorEnvelope {
            error: ErrorPayload {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(payload)).into_response()
    }
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorPayload,
}

#[derive(Debug, Serialize)]
struct ErrorPayload {
    code: &'static str,
    message: String,
}
