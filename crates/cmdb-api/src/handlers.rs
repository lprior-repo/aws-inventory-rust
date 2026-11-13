//! GET handlers. Each clones the parsed view into a serializable DTO.

use crate::dto::{
    HealthResponse, MetaResponse, ResourceDto, ResourcePageResponse, ResourceResponse,
};
use crate::error::AppError;
use crate::query::{self, GraphQuery, LodQuery, ResourceQuery};
use crate::state::AppState;
use crate::summary::{CoverageResponse, GraphResponse, LodResponse};
use axum::Json;
use axum::extract::rejection::{PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse::new(state.fingerprint()))
}

pub async fn meta(State(state): State<AppState>) -> Json<MetaResponse> {
    Json(MetaResponse::from_snapshot(state.snapshot()))
}

pub async fn resources(
    State(state): State<AppState>,
    query: Result<Query<ResourceQuery>, QueryRejection>,
) -> Result<Json<ResourcePageResponse>, AppError> {
    let params = query::into_params(query)?;
    let (filter, request) = query::page_request(&params)?;
    let page = state.snapshot().page(&filter, request)?;
    Ok(Json(ResourcePageResponse::from_page(
        state.fingerprint(),
        &page,
    )))
}

pub async fn resource_by_id(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<ResourceResponse>, AppError> {
    let Path(id) = path.map_err(|_| AppError::malformed_id())?;
    let view = state
        .snapshot()
        .resource(&id)
        .ok_or_else(|| AppError::resource_not_found(&id))?;
    Ok(Json(ResourceResponse {
        fingerprint: state.fingerprint().to_owned(),
        resource: ResourceDto::from(view),
    }))
}

pub async fn graph(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
    query: Result<Query<GraphQuery>, QueryRejection>,
) -> Result<Json<GraphResponse>, AppError> {
    let Path(id) = path.map_err(|_| AppError::malformed_id())?;
    let params = query::into_params(query)?;
    if state.snapshot().resource(&id).is_none() {
        return Err(AppError::resource_not_found(&id));
    }
    let limits = query::graph_limits(&params)?;
    let subgraph = state.snapshot().neighborhood(&id, limits);
    Ok(Json(GraphResponse::from_subgraph(
        state.fingerprint(),
        &subgraph,
        limits,
    )))
}

pub async fn lod(
    State(state): State<AppState>,
    query: Result<Query<LodQuery>, QueryRejection>,
) -> Result<Json<LodResponse>, AppError> {
    let params = query::into_params(query)?;
    let limits = query::lod_limits(&params)?;
    let summary = state.snapshot().lod(limits);
    Ok(Json(LodResponse::from_summary(
        state.fingerprint(),
        &summary,
        limits,
    )))
}

pub async fn coverage(State(state): State<AppState>) -> Json<CoverageResponse> {
    let snapshot = state.snapshot();
    Json(CoverageResponse::from_coverage(
        state.fingerprint(),
        snapshot.coverage(),
    ))
}

pub async fn not_found() -> AppError {
    AppError::not_found("not_found", "no such endpoint")
}

pub async fn method_not_allowed() -> AppError {
    AppError::method_not_allowed()
}
