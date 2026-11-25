use super::super::{name, path};
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::{
    Paginator, Scope, TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector,
};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptGrant",
            "CreateGrant",
            "CreateGrantVersion",
            "CreateLicense",
            "CreateLicenseAssetGroup",
            "CreateLicenseAssetRuleset",
            "CreateLicenseConfiguration",
            "CreateLicenseConversionTaskForResource",
            "CreateLicenseManagerReportGenerator",
            "CreateLicenseVersion",
            "CreateToken",
            "DeleteGrant",
            "DeleteLicense",
            "DeleteLicenseAssetGroup",
            "DeleteLicenseAssetRuleset",
            "DeleteLicenseConfiguration",
            "DeleteLicenseManagerReportGenerator",
            "DeleteToken",
            "ExtendLicenseConsumption",
            "RejectGrant",
            "TagResource",
            "UntagResource",
            "UpdateLicenseAssetGroup",
            "UpdateLicenseAssetRuleset",
            "UpdateLicenseConfiguration",
            "UpdateLicenseManagerReportGenerator",
            "UpdateLicenseSpecificationsForResource",
            "UpdateServiceSettings",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["CheckInLicense"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseConsumptionToken); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["CheckoutBorrowLicense"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseArn, Entitlements, DigitalSignatureMethod, ClientToken); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["CheckoutLicense"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ProductSKU, CheckoutType, KeyFingerprint, Entitlements, ClientToken); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAccessToken", "ListTokens"],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["GetGrant"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (GrantArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetLicenseUsage",
            "ListLicenseVersions",
            "ListReceivedGrantsForOrganization",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetLicenseAssetRuleset"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseAssetRulesetArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListAssociationsForLicenseConfiguration",
            "ListFailuresForLicenseConfigurationOperations",
            "ListUsageForLicenseConfiguration",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseConfigurationArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetLicenseConversionTask"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseConversionTaskId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetServiceSettings",
            "ListDistributedGrants",
            "ListLicenseAssetRulesets",
            "ListLicenseConfigurationsForOrganization",
            "ListLicenseConversionTasks",
            "ListReceivedGrants",
            "ListReceivedLicenses",
            "ListReceivedLicensesForOrganization",
            "ListResourceInventory",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListAssetsForLicenseAssetGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LicenseAssetGroupArn, AssetType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListLicenseSpecificationsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListLicenseConfigurations",
        items: "LicenseConfigurations",
        id: Id::Arn("LicenseConfigurationArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "LicenseConfigurationArn",
            "LicenseConfigurationId",
            "Name",
            "LicenseCountingType",
            "Status",
            "OwnerAccountId",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetLicenseConfiguration",
        items: "$",
        id: Id::Arn("LicenseConfigurationArn"),
        scope: Scope::Region,
        tokens: None,
        query: &[
            "LicenseConfigurationArn",
            "LicenseConfigurationId",
            "Name",
            "LicenseCountingType",
            "Status",
            "OwnerAccountId",
        ],
        tags: Some("Tags"),
        bindings: &[(
            "LicenseConfigurationArn",
            "ListLicenseConfigurations",
            "LicenseConfigurationArn",
        )],
        fixed: &[],
    },
    Resource {
        operation: "ListLicenses",
        items: "Licenses",
        id: Id::Arn("LicenseArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "LicenseArn",
            "LicenseName",
            "ProductName",
            "HomeRegion",
            "Status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListLicenseManagerReportGenerators",
        items: "ReportGenerators",
        id: Id::Arn("LicenseManagerReportGeneratorArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "LicenseManagerReportGeneratorArn",
            "ReportGeneratorName",
            "LastRunStatus",
        ],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListLicenseAssetGroups",
        items: "LicenseAssetGroups",
        id: Id::Arn("LicenseAssetGroupArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["LicenseAssetGroupArn", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetLicense",
        items: "License",
        id: Id::Arn("LicenseArn"),
        scope: Scope::Region,
        tokens: None,
        query: &[
            "LicenseArn",
            "LicenseName",
            "ProductName",
            "HomeRegion",
            "Status",
        ],
        tags: None,
        bindings: &[("LicenseArn", "ListLicenses", "LicenseArn")],
        fixed: &[],
    },
    Resource {
        operation: "GetLicenseAssetGroup",
        items: "LicenseAssetGroup",
        id: Id::Arn("LicenseAssetGroupArn"),
        scope: Scope::Region,
        tokens: None,
        query: &["LicenseAssetGroupArn", "Name", "Status"],
        tags: None,
        bindings: &[(
            "LicenseAssetGroupArn",
            "ListLicenseAssetGroups",
            "LicenseAssetGroupArn",
        )],
        fixed: &[],
    },
    Resource {
        operation: "GetLicenseManagerReportGenerator",
        items: "ReportGenerator",
        id: Id::Arn("LicenseManagerReportGeneratorArn"),
        scope: Scope::Region,
        tokens: None,
        query: &[
            "LicenseManagerReportGeneratorArn",
            "ReportGeneratorName",
            "LastRunStatus",
        ],
        tags: Some("Tags"),
        bindings: &[(
            "LicenseManagerReportGeneratorArn",
            "ListLicenseManagerReportGenerators",
            "LicenseManagerReportGeneratorArn",
        )],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut inventory = compile("license-manager", GROUPS, RESOURCES)?;
    let tags = TagEndpoint {
        parents: vec![TagEndpointParent {
            operation: name("ListLicenseConfigurations")?,
            request: TagEndpointRequest::Member {
                input: path("ResourceArn")?,
                parent: path("LicenseConfigurationArn")?,
            },
        }],
        selector: TagEndpointSelector::List {
            path: path("Tags")?,
            key: path("Key")?,
            value: path("Value")?,
        },
    }
    .operation(Scope::Region, Paginator::None)?;
    inventory
        .operations
        .insert(name("ListTagsForResource")?, tags);
    Ok(inventory)
}
