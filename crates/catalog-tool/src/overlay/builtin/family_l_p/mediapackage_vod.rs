// Pinned Smithy source: .local/aws-models/models/mediapackage-vod/service/2018-11-07/mediapackage-vod-2018-11-07.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "ConfigureLogs",
            "CreateAsset",
            "CreatePackagingConfiguration",
            "CreatePackagingGroup",
            "DeleteAsset",
            "DeletePackagingConfiguration",
            "DeletePackagingGroup",
            "TagResource",
            "UntagResource",
            "UpdatePackagingGroup",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeAsset",
            "DescribePackagingConfiguration",
            "DescribePackagingGroup",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a collection of MediaPackage VOD Asset resources.
    Resource {
        operation: "ListAssets",
        items: "Assets",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "PackagingGroupId", "ResourceId", "SourceArn"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Returns a collection of MediaPackage VOD PackagingConfiguration resources.
    Resource {
        operation: "ListPackagingConfigurations",
        items: "PackagingConfigurations",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "PackagingGroupId"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Returns a collection of MediaPackage VOD PackagingGroup resources.
    Resource {
        operation: "ListPackagingGroups",
        items: "PackagingGroups",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "DomainName", "Id"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediapackage-vod", GROUPS, RESOURCES)
}
