use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "geo-places",
        approved,
        DEFERRED,
        "Place lookup, geocoding, autocomplete and search are billed data-plane products, not account resource enumeration; none is callable.",
        &[(policy::DenyReason::Paid, PAID)],
    )
}

const DEFERRED: &[&str] = &[];

const PAID: &[&str] = &[
    "Autocomplete",
    "Geocode",
    "GetPlace",
    "ReverseGeocode",
    "SearchNearby",
    "SearchText",
    "Suggest",
];
