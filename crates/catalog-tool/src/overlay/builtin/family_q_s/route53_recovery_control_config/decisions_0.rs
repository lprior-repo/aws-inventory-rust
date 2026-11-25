use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateCluster", Decision::Denied(DenyReason::Mutating)),
    ("CreateControlPanel", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateRoutingControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSafetyRule", Decision::Denied(DenyReason::Mutating)),
    ("DeleteCluster", Decision::Denied(DenyReason::Mutating)),
    ("DeleteControlPanel", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteRoutingControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSafetyRule", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeCluster",
        Decision::Deferred(
            "Required selectors ClusterArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Cluster. Pinned purpose: Display the details about a cluster. The response includes the cluster name, endpoints, status, and Amazon Resource Name (ARN).",
        ),
    ),
    (
        "DescribeControlPanel",
        Decision::Deferred(
            "Required selectors ControlPanelArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ControlPanel. Pinned purpose: Displays details about a control panel.",
        ),
    ),
    (
        "DescribeRoutingControl",
        Decision::Deferred(
            "Required selectors RoutingControlArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RoutingControl. Pinned purpose: Displays details about a routing control. A routing control has one of two states: ON and OFF. You can map the routing control state to the state of an Amazon Route 53 health check, which can be used to control routing. To get or update the routing c",
        ),
    ),
    (
        "DescribeSafetyRule",
        Decision::Deferred(
            "Required selectors SafetyRuleArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AssertionRule, GatingRule. Pinned purpose: Returns information about a safety rule.",
        ),
    ),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: Get information about the resource policy for a cluster.",
        ),
    ),
    (
        "ListAssociatedRoute53HealthChecks",
        Decision::Deferred(
            "Required selectors RoutingControlArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HealthCheckIds, NextToken. Pinned purpose: Returns an array of all Amazon Route 53 health checks associated with a specific routing control.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags for a resource.",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCluster", Decision::Denied(DenyReason::Mutating)),
    ("UpdateControlPanel", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateRoutingControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSafetyRule", Decision::Denied(DenyReason::Mutating)),
];
