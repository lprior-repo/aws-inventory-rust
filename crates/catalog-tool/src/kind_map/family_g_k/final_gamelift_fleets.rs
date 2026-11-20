use crate::kind_map::{CfnMapping, OperationMapping};

// Pinned cloudcontrol-universal.json includes these exact resource types. Each listed operation
// returns the modeled identifier directly; no synthetic ARN or API-only alias is needed.
pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::GameLift::ContainerFleet"),
    CfnMapping::canonical("AWS::GameLift::ContainerGroupDefinition"),
    CfnMapping::canonical("AWS::GameLift::MatchmakingConfiguration"),
    CfnMapping::canonical("AWS::GameLift::MatchmakingRuleSet"),
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "gamelift",
        operation: "ListContainerFleets",
        kind: "AWS::GameLift::ContainerFleet",
    },
    OperationMapping {
        service: "gamelift",
        operation: "ListContainerGroupDefinitions",
        kind: "AWS::GameLift::ContainerGroupDefinition",
    },
    OperationMapping {
        service: "gamelift",
        operation: "DescribeMatchmakingConfigurations",
        kind: "AWS::GameLift::MatchmakingConfiguration",
    },
    OperationMapping {
        service: "gamelift",
        operation: "DescribeMatchmakingRuleSets",
        kind: "AWS::GameLift::MatchmakingRuleSet",
    },
];
