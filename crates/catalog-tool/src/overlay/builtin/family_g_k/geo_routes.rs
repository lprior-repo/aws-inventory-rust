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

const DEFERRED: &[&str] = &[];

const PAID: &[&str] = &[
    "CalculateIsolines",
    "CalculateRouteMatrix",
    "CalculateRoutes",
    "OptimizeWaypoints",
    "SnapToRoads",
];
