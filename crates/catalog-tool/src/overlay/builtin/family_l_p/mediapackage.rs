// Pinned Smithy source: .local/aws-models/models/mediapackage/service/2017-10-12/mediapackage-2017-10-12.json
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
            "CreateChannel",
            "CreateHarvestJob",
            "CreateOriginEndpoint",
            "DeleteChannel",
            "DeleteOriginEndpoint",
            "RotateChannelCredentials",
            "RotateIngestEndpointCredentials",
            "TagResource",
            "UntagResource",
            "UpdateChannel",
            "UpdateOriginEndpoint",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeChannel", "ListChannels"],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["DescribeHarvestJob", "DescribeOriginEndpoint"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListHarvestJobs"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
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
    // Returns a collection of OriginEndpoint records.
    Resource {
        operation: "ListOriginEndpoints",
        items: "OriginEndpoints",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ChannelId", "Id", "ManifestName"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediapackage", GROUPS, RESOURCES)
}
