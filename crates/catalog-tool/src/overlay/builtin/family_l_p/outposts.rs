// Pinned Smithy source: .local/aws-models/models/outposts/service/2019-12-03/outposts-2019-12-03.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CancelCapacityTask", "CancelOrder", "CreateOrder", "CreateOutpost", 
            "CreatePrivateConnectivityConfig", "CreateQuote", "CreateRenewal", "CreateSite", 
            "DeleteOutpost", "DeleteQuote", "DeleteSite", "StartCapacityTask", 
            "StartConnection", "StartOutpostDecommission", "TagResource", "UntagResource", 
            "UpdateOutpost", "UpdateQuote", "UpdateSite", "UpdateSiteAddress", 
            "UpdateSiteRackPhysicalProperties",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetCapacityTask",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (CapacityTaskId, OutpostIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetCatalogItem",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (CatalogItemId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetConnection",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ConnectionId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetOrder",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (OrderId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
             "GetOutpostInstanceTypes", "GetPrivateConnectivityConfig",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (OutpostId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetOutpostBillingInformation", "GetOutpostSupportedInstanceTypes", 
            "GetRenewalPricing", "ListAssetInstances", "ListAssets",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (OutpostIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetQuote",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (QuoteIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetSiteAddress",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (SiteId, AddressType); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListBlockingInstancesForCapacityTask",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (OutpostIdentifier, CapacityTaskId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListCapacityTasks", "ListCatalogItems", "ListOrderableInstanceTypes", 
            "ListOrders", "ListQuotes",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists the Outposts for your Amazon Web Services account. Use filters to return specific results. If you specify multiple filters, the results include only the resources that match 
    Resource {
        operation: "ListOutposts",
        items: "Outposts", id: Id::Arn("OutpostArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["OutpostArn", "OutpostId", "OwnerId", "SiteId", "Name", "LifeCycleStatus", "AvailabilityZoneId", "SiteArn", "SupportedHardwareType", "RackScalingType"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists the Outpost sites for your Amazon Web Services account. Use filters to return specific results. Use filters to return specific results. If you specify multiple filters, the r
    Resource {
        operation: "ListSites",
        items: "Sites", id: Id::Arn("SiteArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["SiteArn", "SiteId", "AccountId", "Name", "OperatingAddressStateOrRegion"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Same pinned response resource shape as ListOutposts; no content-bearing expansion.
    Resource {
        operation: "GetOutpost",
        items: "Outpost", id: Id::Arn("OutpostArn"),
        scope: Scope::Region, tokens: None,
        query: &["OutpostArn", "OutpostId", "OwnerId", "SiteId", "Name", "LifeCycleStatus", "AvailabilityZoneId", "SiteArn", "SupportedHardwareType", "RackScalingType"],
        tags: Some("Tags"),
        bindings: &[("OutpostId", "ListOutposts", "OutpostId")],
        fixed: &[],
    },
    // Same pinned response resource shape as ListSites; no content-bearing expansion.
    Resource {
        operation: "GetSite",
        items: "Site", id: Id::Arn("SiteArn"),
        scope: Scope::Region, tokens: None,
        query: &["SiteArn", "SiteId", "AccountId", "Name", "OperatingAddressStateOrRegion"],
        tags: Some("Tags"),
        bindings: &[("SiteId", "ListSites", "SiteId")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("outposts", GROUPS, RESOURCES)
}
