use crate::kind_map::{CfnMapping, OperationMapping};

// Pinned cloudcontrol-universal.json includes this exact resource type. ListContainerFleets
// returns its modeled FleetArn directly; no synthetic ARN or API-only alias is needed.
pub(super) const CFN: &[CfnMapping<'static>] =
    &[CfnMapping::canonical("AWS::GameLift::ContainerFleet")];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[OperationMapping {
    service: "gamelift",
    operation: "ListContainerFleets",
    kind: "AWS::GameLift::ContainerFleet",
}];
