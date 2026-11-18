// Pinned Smithy source: .local/aws-models/models/partnercentral-revenue-measurement/service/2022-07-26/partnercentral-revenue-measurement-2022-07-26.json
use super::policy::{Group, Id, Input, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateMarketplaceRevenueShare",
            "CreateMarketplaceRevenueShareAllocation",
            "CreateRevenueAttribution",
            "StartRevenueAttributionAllocationsTask",
            "TagResource",
            "UntagResource",
            "UpdateMarketplaceRevenueShareAllocation",
            "UpdateRevenueAttribution",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetMarketplaceRevenueShare",
            "ListMarketplaceRevenueShareAllocations",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Catalog, ProductId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetMarketplaceRevenueShareAllocation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Catalog, ProductId, MarketplaceRevenueShareAllocationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRevenueAttribution"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Catalog, Identifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRevenueAttributionAllocation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Catalog, RevenueAttributionIdentifier, RevenueAttributionAllocationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetRevenueAttributionAllocationsTask",
            "ListRevenueAttributionAllocations",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Catalog, RevenueAttributionIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a paginated list of revenue attributions with optional filters.
    Resource {
        operation: "ListRevenueAttributions",
        items: "RevenueAttributionSummaries",
        id: Id::Arn("Arn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[Input::String("Catalog", "AWS")],
    },
    // Returns a paginated list of marketplace revenue shares with optional filters.
    Resource {
        operation: "ListMarketplaceRevenueShares",
        items: "MarketplaceRevenueShareSummaries",
        id: Id::Arn("Arn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ProductId", "ProductName"],
        tags: None,
        bindings: &[],
        fixed: &[Input::String("Catalog", "AWS")],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("partnercentral-revenue-measurement", GROUPS, RESOURCES)
}
