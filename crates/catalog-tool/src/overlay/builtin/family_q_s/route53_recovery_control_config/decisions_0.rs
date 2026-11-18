// Pinned operation purposes: .local/aws-models/models/route53-recovery-control-config/service/2020-11-02/route53-recovery-control-config-2020-11-02.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Create a new cluster. A cluster is a set of redundant Regional endpoints against which you can run API calls to update or get the state of one or more routing controls. Each cluster has a name, status, Amazon Resource Name (ARN), and an array of the five cluster endpoints (one for each supported Amazon Web Services Region) that you can use with API calls to the cluster data plane.
    ("CreateCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new control panel. A control panel represents a group of routing controls that can be changed together in a single transaction. You can use a control panel to centrally view the operational status of applications across your organization, and trigger multi-app failovers in a single transaction, for example, to fail over an Availability Zone or Amazon Web Services Region.
    ("CreateControlPanel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new routing control. A routing control has one of two states: ON and OFF. You can map the routing control state to the state of an Amazon Route 53 health check, which can be used to control traffic routing. To get or update the routing control state, see the Recovery Cluster (data plane) API actions for Amazon Route 53 Application Recovery Controller.
    (
        "CreateRoutingControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a safety rule in a control panel. Safety rules let you add safeguards around changing routing control states, and for enabling and disabling routing controls, to help prevent unexpected outcomes. There are two types of safety rules: assertion rules and gating rules. Assertion rule: An assertion rule enforces that, when you change a routing control state, that a certain criteria is met. For
    ("CreateSafetyRule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a cluster.
    ("DeleteCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a control panel.
    ("DeleteControlPanel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a routing control.
    (
        "DeleteRoutingControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a safety rule. /&gt;
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds a tag to a resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing cluster. You can only update the network type of a cluster.
    ("UpdateCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a control panel. The only update you can make to a control panel is to change the name of the control panel.
    ("UpdateControlPanel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a routing control. You can only update the name of the routing control. To get or update the routing control state, see the Recovery Cluster (data plane) API actions for Amazon Route 53 Application Recovery Controller.
    (
        "UpdateRoutingControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update a safety rule (an assertion rule or gating rule). You can only update the name and the waiting period for a safety rule. To make other updates, delete the safety rule and create a new one.
    ("UpdateSafetyRule", Decision::Denied(DenyReason::Mutating)),
];
