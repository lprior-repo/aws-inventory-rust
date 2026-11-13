//! Query/body bounds shared by every route.

use crate::{AppError, MAX_BODY_BYTES, MAX_QUERY_BYTES};
use axum::extract::Request;
use axum::http::HeaderMap;
use axum::http::header::CONTENT_LENGTH;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Reject oversized query strings and declared bodies before routing.
pub async fn enforce(request: Request, next: Next) -> Response {
    let query_len = request.uri().query().map_or(0, str::len);
    if query_len > MAX_QUERY_BYTES {
        return AppError::query_too_long(MAX_QUERY_BYTES).into_response();
    }
    if declared_body(request.headers()) > MAX_BODY_BYTES {
        return AppError::payload_too_large(MAX_BODY_BYTES).into_response();
    }
    next.run(request).await
}

fn declared_body(headers: &HeaderMap) -> usize {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}
