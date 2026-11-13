//! Read-only local HTTP query API over a parsed [`cmdb_view::Snapshot`].
//!
//! The server loads one export, fingerprints it, and answers deterministic GET
//! queries from memory. There is no authentication, no write path and no
//! database; the default bind is loopback-only and non-local binds are rejected
//! by [`Config`] until an authentication story exists.
//!
//! Every response body carries the snapshot fingerprint so callers can detect a
//! stale parse without comparing whole inventories.

#![forbid(unsafe_code)]
#![deny(unused_must_use)]

mod config;
mod dto;
mod error;
mod handlers;
mod limits;
mod query;
mod server;
mod state;
mod summary;

pub use config::{Config, ConfigError, DEFAULT_BIND, USAGE};
pub use error::AppError;
pub use server::{StartupError, report, run};
pub use state::AppState;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::Method;
use axum::middleware;
use axum::routing::get;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

/// Default resource page size when `limit` is absent.
pub const DEFAULT_PAGE_LIMIT: usize = 100;
/// Default graph node cap when `max_nodes` is absent.
pub const DEFAULT_GRAPH_NODES: usize = 1_000;
/// Default graph edge cap when `max_edges` is absent.
pub const DEFAULT_GRAPH_EDGES: usize = 2_000;
/// Largest accepted query string; longer queries are refused with 400.
pub const MAX_QUERY_BYTES: usize = 4_096;
/// Largest accepted request body; GET handlers never read a body.
pub const MAX_BODY_BYTES: usize = 1_024;

/// Build the complete read-only router over `state`.
///
/// Every route is GET; CORS advertises only GET (plus its preflight), and the
/// fallbacks return typed JSON errors instead of HTML or plain text.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/meta", get(handlers::meta))
        .route("/api/v1/resources", get(handlers::resources))
        .route("/api/v1/resources/{id}", get(handlers::resource_by_id))
        .route("/api/v1/graph/{id}", get(handlers::graph))
        .route("/api/v1/lod", get(handlers::lod))
        .route("/api/v1/coverage", get(handlers::coverage))
        .fallback(handlers::not_found)
        .method_not_allowed_fallback(handlers::method_not_allowed)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn(limits::enforce))
        .layer(cors_layer())
        .with_state(state)
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any)
        .max_age(Duration::from_secs(600))
}
