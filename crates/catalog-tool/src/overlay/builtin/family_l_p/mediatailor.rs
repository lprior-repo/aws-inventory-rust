// Pinned Smithy source: .local/aws-models/models/mediatailor/service/2018-04-23/mediatailor-2018-04-23.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "ConfigureLogsForChannel", "ConfigureLogsForPlaybackConfiguration", 
            "CreateChannel", "CreateLiveSource", "CreatePrefetchSchedule", "CreateProgram", 
            "CreateSourceLocation", "CreateVodSource", "DeleteChannel", "DeleteChannelPolicy", 
            "DeleteFunction", "DeleteLiveSource", "DeletePlaybackConfiguration", 
            "DeletePrefetchSchedule", "DeleteProgram", "DeleteSourceLocation", 
            "DeleteVodSource", "PutChannelPolicy", "PutFunction", "PutPlaybackConfiguration", 
            "StartChannel", "StopChannel", "TagResource", "UntagResource", "UpdateChannel", 
            "UpdateLiveSource", "UpdateProgram", "UpdateSourceLocation", "UpdateVodSource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeChannel", "GetChannelPolicy", "GetChannelSchedule",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ChannelName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeLiveSource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (LiveSourceName, SourceLocationName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeProgram",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ChannelName, ProgramName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeSourceLocation", "ListLiveSources", "ListVodSources",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (SourceLocationName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeVodSource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (SourceLocationName, VodSourceName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetFunction",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (FunctionId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetPlaybackConfiguration",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Name); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetPrefetchSchedule",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Name, PlaybackConfigurationName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAlerts", "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListFunctions", "ListPlaybackConfigurations",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListPrefetchSchedules",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PlaybackConfigurationName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Retrieves information about the channels that are associated with the current AWS account.
    Resource {
        operation: "ListChannels",
        items: "Items", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ChannelName", "ChannelState"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists the source locations for a channel. A source location defines the host server URL, and contains a list of sources.
    Resource {
        operation: "ListSourceLocations",
        items: "Items", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "SourceLocationName"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediatailor", GROUPS, RESOURCES)
}
