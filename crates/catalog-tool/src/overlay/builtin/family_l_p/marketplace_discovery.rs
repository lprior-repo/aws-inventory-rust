use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "GetListing",
            "GetOffer",
            "GetOfferSet",
            "GetOfferTerms",
            "GetProduct",
            "ListFulfillmentOptions",
            "ListPurchaseOptions",
        ],
        class: OverlayClass::Deny(DenyReason::ReferenceData),
        reason: None,
    },
    Group {
        operations: &["SearchFacets", "SearchListings"],
        class: OverlayClass::Defer,
        reason: Some(
            "Product and offer catalog is reference data, not caller-owned resource inventory.",
        ),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-discovery", GROUPS, &[])
}
