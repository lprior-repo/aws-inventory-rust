use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "geo-maps",
        approved,
        DEFERRED,
        "Map glyphs, sprites, static images, styles and tiles are billed data-plane products, not account resource enumeration; none is callable.",
        &[(policy::DenyReason::Paid, PAID)],
    )
}

const DEFERRED: &[&str] = &[];

const PAID: &[&str] = &[
    "GetGlyphs",
    "GetSprites",
    "GetStaticMap",
    "GetStyleDescriptor",
    "GetTile",
];
