//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/geo-routes/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "geo-routes",
        approved,
        DEFERRED,
        "Route, isoline, matrix, waypoint and GPS computations are billed data-plane products, not account resource enumeration; none is callable.",
        &[(policy::DenyReason::Paid, PAID)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[];

const PAID: &[&str] = &[
    "CalculateIsolines",
    "CalculateRouteMatrix",
    "CalculateRoutes",
    "OptimizeWaypoints",
    "SnapToRoads",
];
