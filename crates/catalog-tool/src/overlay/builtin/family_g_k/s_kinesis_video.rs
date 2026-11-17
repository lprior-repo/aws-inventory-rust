// Pinned source: .local/aws-models/models/kinesis-video/service/2017-09-30/kinesis-video-2017-09-30.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "Connect", "CreateSignalingChannel", "CreateStream",
            "DeleteSignalingChannel", "DeleteStream", "GetSignalingChannelEndpoint",
            "GetSignalingConfiguration", "GetDataEndpoint", "GetIceServerEndpoint",
            "ListStreamFragments", "ListPlaybackEndpoints", "PutKeyPair", "TagResource",
            "UntagResource", "UpdateSignalingChannel", "UpdateStream",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeSignalingChannel", "DescribeStream", "ListTagsForResource",
            "ListEdgeAgentConfigurations",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Arn, HubDeviceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListStreams",
        items: "StreamInfoList", id: Id::Key(&["StreamARN"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["StreamARN", "StreamName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListSignalingChannels",
        items: "ChannelInfoList", id: Id::Key(&["ChannelARN"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ChannelARN", "ChannelName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "DescribeMappedResourceConfiguration",
        items: "MappedResourceConfigurationList", id: Id::Key(&["ResourceArn"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ResourceArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("kinesis-video", GROUPS, RESOURCES)
}