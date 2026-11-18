// Pinned Smithy source: .local/aws-models/models/mediapackagev2/service/2022-12-25/mediapackagev2-2022-12-25.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CancelHarvestJob",
            "CreateChannel",
            "CreateChannelGroup",
            "CreateHarvestJob",
            "CreateOriginEndpoint",
            "DeleteChannel",
            "DeleteChannelGroup",
            "DeleteChannelPolicy",
            "DeleteOriginEndpoint",
            "DeleteOriginEndpointPolicy",
            "PutChannelPolicy",
            "PutOriginEndpointPolicy",
            "ResetChannelState",
            "ResetOriginEndpointState",
            "TagResource",
            "UntagResource",
            "UpdateChannel",
            "UpdateChannelGroup",
            "UpdateOriginEndpoint",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetChannel", "GetChannelPolicy"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelGroupName, ChannelName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetChannelGroup", "ListHarvestJobs"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelGroupName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetHarvestJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelGroupName, ChannelName, OriginEndpointName, HarvestJobName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetOriginEndpoint", "GetOriginEndpointPolicy"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelGroupName, ChannelName, OriginEndpointName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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
    // Retrieves all channel groups that are configured in Elemental MediaPackage.
    Resource {
        operation: "ListChannelGroups",
        items: "Items",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ChannelGroupName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves all channels in a specific channel group that are configured in AWS Elemental MediaPackage.
    Resource {
        operation: "ListChannels",
        items: "Items",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ChannelName", "ChannelGroupName", "InputType"],
        tags: None,
        bindings: &[("ChannelGroupName", "ListChannelGroups", "ChannelGroupName")],
        fixed: &[],
    },
    // Retrieves all origin endpoints in a specific channel that are configured in AWS Elemental MediaPackage.
    Resource {
        operation: "ListOriginEndpoints",
        items: "Items",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "Arn",
            "ChannelGroupName",
            "ChannelName",
            "OriginEndpointName",
            "ContainerType",
        ],
        tags: None,
        bindings: &[
            ("ChannelGroupName", "ListChannels", "ChannelGroupName"),
            ("ChannelName", "ListChannels", "ChannelName"),
        ],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediapackagev2", GROUPS, RESOURCES)
}
