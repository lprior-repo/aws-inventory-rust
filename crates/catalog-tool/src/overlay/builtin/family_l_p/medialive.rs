// Pinned Smithy source: .local/aws-models/models/medialive/service/2017-10-14/medialive-2017-10-14.json
use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptInputDeviceTransfer",
            "BatchDelete",
            "BatchStart",
            "BatchStop",
            "BatchUpdateSchedule",
            "CancelInputDeviceTransfer",
            "ClaimDevice",
            "CreateChannel",
            "CreateChannelPlacementGroup",
            "CreateCloudWatchAlarmTemplate",
            "CreateCloudWatchAlarmTemplateGroup",
            "CreateCluster",
            "CreateEventBridgeRuleTemplate",
            "CreateEventBridgeRuleTemplateGroup",
            "CreateInput",
            "CreateInputSecurityGroup",
            "CreateMultiplex",
            "CreateMultiplexProgram",
            "CreateNetwork",
            "CreateNode",
            "CreateNodeRegistrationScript",
            "CreatePartnerInput",
            "CreateSdiSource",
            "CreateSignalMap",
            "CreateTags",
            "DeleteChannel",
            "DeleteChannelPlacementGroup",
            "DeleteCloudWatchAlarmTemplate",
            "DeleteCloudWatchAlarmTemplateGroup",
            "DeleteCluster",
            "DeleteEventBridgeRuleTemplate",
            "DeleteEventBridgeRuleTemplateGroup",
            "DeleteInput",
            "DeleteInputSecurityGroup",
            "DeleteMultiplex",
            "DeleteMultiplexProgram",
            "DeleteNetwork",
            "DeleteNode",
            "DeleteReservation",
            "DeleteSchedule",
            "DeleteSdiSource",
            "DeleteSignalMap",
            "DeleteTags",
            "PurchaseOffering",
            "RebootInputDevice",
            "RejectInputDeviceTransfer",
            "RestartChannelPipelines",
            "StartChannel",
            "StartDeleteMonitorDeployment",
            "StartInputDevice",
            "StartInputDeviceMaintenanceWindow",
            "StartMonitorDeployment",
            "StartMultiplex",
            "StartUpdateSignalMap",
            "StopChannel",
            "StopInputDevice",
            "StopMultiplex",
            "TransferInputDevice",
            "UpdateAccountConfiguration",
            "UpdateChannel",
            "UpdateChannelClass",
            "UpdateChannelPlacementGroup",
            "UpdateCloudWatchAlarmTemplate",
            "UpdateCloudWatchAlarmTemplateGroup",
            "UpdateCluster",
            "UpdateEventBridgeRuleTemplate",
            "UpdateEventBridgeRuleTemplateGroup",
            "UpdateInput",
            "UpdateInputDevice",
            "UpdateInputSecurityGroup",
            "UpdateMultiplex",
            "UpdateMultiplexProgram",
            "UpdateNetwork",
            "UpdateNode",
            "UpdateNodeState",
            "UpdateReservation",
            "UpdateSdiSource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeAccountConfiguration",
            "ListOfferings",
            "ListReservations",
            "ListVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["DescribeChannel", "DescribeSchedule", "ListAlerts"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeChannelPlacementGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelPlacementGroupId, ClusterId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeCluster", "ListClusterAlerts"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ClusterId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeInput"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InputId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeInputDevice"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InputDeviceId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeInputDeviceThumbnail"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InputDeviceId, Accept); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeInputSecurityGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InputSecurityGroupId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "DescribeMultiplex",
            "ListMultiplexAlerts",
            "ListMultiplexPrograms",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (MultiplexId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeMultiplexProgram"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (MultiplexId, ProgramName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeNetwork"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (NetworkId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeNode"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ClusterId, NodeId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeOffering"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (OfferingId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeReservation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ReservationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeSdiSource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (SdiSourceId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeThumbnails"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChannelId, PipelineId, ThumbnailType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetCloudWatchAlarmTemplate",
            "GetCloudWatchAlarmTemplateGroup",
            "GetEventBridgeRuleTemplate",
            "GetEventBridgeRuleTemplateGroup",
            "GetSignalMap",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Identifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListInputDeviceTransfers"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TransferType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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

#[path = "medialive_resources.rs"]
mod resources;
use resources::RESOURCES;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("medialive", GROUPS, RESOURCES)
}
