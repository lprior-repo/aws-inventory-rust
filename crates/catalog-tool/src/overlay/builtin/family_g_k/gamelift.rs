//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/gamelift/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{FixedValue, NameError, OperationOverlay, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListAliases",
            policy::query(
                policy::enumerate(
                    "Aliases",
                    policy::arn("AliasArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["AliasArn", "RoutingStrategy.FleetId"],
            )?,
        ),
        (
            "ListBuilds",
            policy::query(
                policy::enumerate(
                    "Builds",
                    policy::arn("BuildArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["BuildArn"],
            )?,
        ),
        (
            "ListScripts",
            policy::query(
                policy::enumerate(
                    "Scripts",
                    policy::arn("ScriptArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["ScriptArn", "ScriptId", "StorageLocation"],
            )?,
        ),
        // Pinned ListContainerFleets has no required filter: omitting ContainerGroupDefinitionName
        // enumerates all owned fleets in their home request region. Full ContainerFleet closure
        // contains configuration/ports/CIDRs, limits, status, deployment IDs and remote locations;
        // IAM/container-definition ARNs and log bucket/group names are references, not credentials,
        // definition bytes, logs or player/session content. Never follow these into detail reads.
        (
            "ListContainerFleets",
            policy::query(
                policy::enumerate(
                    "ContainerFleets",
                    policy::arn("FleetArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &[
                    "FleetArn",
                    "FleetId",
                    "FleetRoleArn",
                    "GameServerContainerGroupDefinitionArn",
                    "PerInstanceContainerGroupDefinitionArn",
                ],
            )?,
        ),
        (
            "ListLocations",
            OperationOverlay {
                fixed: [(
                    super::super::path("Filters")?,
                    FixedValue::Strings(vec!["CUSTOM".to_owned()]),
                )]
                .into_iter()
                .collect(),
                ..policy::query(
                    policy::enumerate(
                        "Locations",
                        policy::arn("LocationArn")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    &["LocationArn", "LocationName"],
                )?
            },
        ),
        (
            "DescribeFleetAttributes",
            policy::query(
                policy::enumerate(
                    "FleetAttributes",
                    policy::arn("FleetArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["FleetArn", "FleetId", "BuildArn"],
            )?,
        ),
        (
            "ListGameServerGroups",
            policy::query(
                policy::enumerate(
                    "GameServerGroups",
                    policy::arn("GameServerGroupArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["GameServerGroupArn"],
            )?,
        ),
        (
            "DescribeAlias",
            policy::query(
                policy::bind(
                    policy::enumerate("Alias", policy::arn("AliasArn")?, None)?,
                    "AliasId",
                    "ListAliases",
                    "AliasId",
                )?,
                &["AliasArn"],
            )?,
        ),
        (
            "DescribeBuild",
            policy::query(
                policy::bind(
                    policy::enumerate("Build", policy::arn("BuildArn")?, None)?,
                    "BuildId",
                    "ListBuilds",
                    "BuildId",
                )?,
                &["BuildArn"],
            )?,
        ),
        (
            "ListContainerGroupDefinitions",
            policy::query(
                policy::enumerate(
                    "ContainerGroupDefinitions",
                    policy::arn("ContainerGroupDefinitionArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["ContainerGroupDefinitionArn"],
            )?,
        ),
        (
            "DescribeMatchmakingConfigurations",
            policy::query(
                policy::enumerate(
                    "Configurations",
                    policy::arn("ConfigurationArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["ConfigurationArn"],
            )?,
        ),
        (
            "DescribeMatchmakingRuleSets",
            policy::query(
                policy::enumerate(
                    "RuleSets",
                    policy::arn("RuleSetArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["RuleSetArn"],
            )?,
        ),
    ];
    policy::finish(
        "gamelift",
        approved,
        DEFERRED,
        "Builds, aliases, fleets (including unfiltered owned container fleets), FleetIQ groups, Realtime script metadata and CUSTOM-only locations are approved. ContainerFleet nesting is configuration metadata and references, not definition/log bytes or credentials. Script storage and ping beacons contain references, not bytes or credentials. Provider AWS locations, game-session content, credentials and log URLs are not collected.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeCompute",
    "DescribeContainerFleet",
    "DescribeContainerGroupDefinition",
    "DescribeContainerGroupPortMappings",
    "DescribeEC2InstanceLimits",
    "DescribeFleetCapacity",
    "DescribeFleetDeployment",
    "DescribeFleetLocationAttributes",
    "DescribeFleetLocationCapacity",
    "DescribeFleetLocationUtilization",
    "DescribeFleetPortSettings",
    "DescribeFleetUtilization",
    "DescribeGameServer",
    "DescribeGameServerGroup",
    "DescribeGameServerInstances",
    "DescribeGameSessionPlacement",
    "DescribeInstances",
    "DescribeRuntimeConfiguration",
    "DescribeScalingPolicies",
    "DescribeScript",
    "DescribeVpcPeeringAuthorizations",
    "DescribeVpcPeeringConnections",
    "GetPlayerConnectionDetails",
    "ListCompute",
    "ListContainerGroupDefinitionVersions",
    "ListFleetDeployments",
    "ListFleets",
    "ListGameServers",
    "ListTagsForResource",
    "ResolveAlias",
    "SearchGameSessions",
    "ValidateMatchmakingRuleSet",
];

const MUTATING: &[&str] = &[
    "AcceptMatch",
    "ClaimGameServer",
    "CreateAlias",
    "CreateBuild",
    "CreateContainerFleet",
    "CreateContainerGroupDefinition",
    "CreateFleet",
    "CreateFleetLocations",
    "CreateGameServerGroup",
    "CreateGameSession",
    "CreateGameSessionQueue",
    "CreateLocation",
    "CreateMatchmakingConfiguration",
    "CreateMatchmakingRuleSet",
    "CreatePlayerSession",
    "CreatePlayerSessions",
    "CreateScript",
    "CreateVpcPeeringAuthorization",
    "CreateVpcPeeringConnection",
    "DeleteAlias",
    "DeleteBuild",
    "DeleteContainerFleet",
    "DeleteContainerGroupDefinition",
    "DeleteFleet",
    "DeleteFleetLocations",
    "DeleteGameServerGroup",
    "DeleteGameSessionQueue",
    "DeleteLocation",
    "DeleteMatchmakingConfiguration",
    "DeleteMatchmakingRuleSet",
    "DeleteScalingPolicy",
    "DeleteScript",
    "DeleteVpcPeeringAuthorization",
    "DeleteVpcPeeringConnection",
    "DeregisterCompute",
    "DeregisterGameServer",
    "PutScalingPolicy",
    "RegisterCompute",
    "RegisterGameServer",
    "ResumeGameServerGroup",
    "StartFleetActions",
    "StartGameSessionPlacement",
    "StartMatchBackfill",
    "StartMatchmaking",
    "StopFleetActions",
    "StopGameSessionPlacement",
    "StopMatchmaking",
    "SuspendGameServerGroup",
    "TagResource",
    "TerminateGameSession",
    "UntagResource",
    "UpdateAlias",
    "UpdateBuild",
    "UpdateContainerFleet",
    "UpdateContainerGroupDefinition",
    "UpdateFleetAttributes",
    "UpdateFleetCapacity",
    "UpdateFleetPortSettings",
    "UpdateGameServer",
    "UpdateGameServerGroup",
    "UpdateGameSession",
    "UpdateGameSessionQueue",
    "UpdateMatchmakingConfiguration",
    "UpdateRuntimeConfiguration",
    "UpdateScript",
];

const CONTENT: &[&str] = &[
    "DescribeFleetEvents",
    "DescribeGameSessionDetails",
    "DescribeGameSessionQueues",
    "DescribeGameSessions",
    "DescribeMatchmaking",
    "DescribePlayerSessions",
    "GetGameSessionLogUrl",
];

const SECRET: &[&str] = &[
    "GetComputeAccess",
    "GetComputeAuthToken",
    "GetInstanceAccess",
    "RequestUploadCredentials",
];
