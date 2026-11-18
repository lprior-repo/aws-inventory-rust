use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeSecurityGroupRules", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecurityGroupRules")?),
        identity: Some(Identity::Key(vec![path("SecurityGroupRuleId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecurityGroupVpcAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecurityGroupVpcAssociations")?),
        identity: Some(Identity::Key(vec![path("GroupId")?, path("VpcId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeServiceLinkVirtualInterfaces", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ServiceLinkVirtualInterfaces")?),
        identity: Some(Identity::Key(vec![path("ServiceLinkVirtualInterfaceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSnapshotAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires SnapshotId and Attribute; createVolumePermission and productCodes require enum-input fanout and parent snapshot identity, not a single arbitrarily selected attribute.".to_owned()),
        ..empty()
    }),
("DescribeSnapshotTierStatus", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SnapshotTierStatuses")?),
        identity: Some(Identity::Key(vec![path("SnapshotId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSnapshots", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Snapshots")?),
        fixed: [(path("OwnerIds")?, FixedValue::Strings(vec!["self".to_owned()]))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("SnapshotId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSpotDatafeedSubscription", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("SpotDatafeedSubscription.OwnerId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeSpotFleetInstances", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required SpotFleetRequestId has only a content-bearing parent DescribeSpotFleetRequests, which remains denied because LaunchSpecifications[].UserData is returned; needs a safe parent-ID projection before expansion.".to_owned()),
        ..empty()
    }),
("DescribeSpotFleetRequestHistory", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required SpotFleetRequestId has a denied content-bearing parent and StartTime requires an explicit history window; HistoryRecords[] also lacks a stable event ID.".to_owned()),
        ..empty()
    }),
("DescribeSpotFleetRequests", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeSpotInstanceRequests", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeSpotPriceHistory", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeStaleSecurityGroups", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("StaleSecurityGroupSet")?),
        identity: Some(Identity::Key(vec![path("GroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("VpcId")?, Binding::From { operation: name("DescribeVpcs")?, path: path("VpcId")? })].into_iter().collect(),
        ..empty()
    }),
("DescribeStoreImageTasks", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("StoreImageTaskResults[] can contain multiple tasks per AmiId with Bucket, TaskStartTime and S3ObjectKey; requires task-history identity instead of collapsing distinct tasks under the AMI ID.".to_owned()),
        ..empty()
    }),
("DescribeTags", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Tags")?),
        identity: Some(Identity::Key(vec![path("ResourceId")?, path("Key")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTrafficMirrorFilterRules", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TrafficMirrorFilterRules")?),
        identity: Some(Identity::Key(vec![path("TrafficMirrorFilterRuleId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTrafficMirrorFilters", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TrafficMirrorFilters")?),
        identity: Some(Identity::Key(vec![path("TrafficMirrorFilterId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTrafficMirrorSessions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TrafficMirrorSessions")?),
        identity: Some(Identity::Key(vec![path("TrafficMirrorSessionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTrafficMirrorTargets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TrafficMirrorTargets")?),
        identity: Some(Identity::Key(vec![path("TrafficMirrorTargetId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayAttachments", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayAttachments")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayAttachmentId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    })
    ])
}
