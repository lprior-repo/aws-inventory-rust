// Pinned operation purposes: .local/aws-models/models/route53-recovery-cluster/service/2019-12-02/route53-recovery-cluster-2019-12-02.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("GetRoutingControlState", Decision::Deferred("Required selectors RoutingControlArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RoutingControlArn, RoutingControlState, RoutingControlName. Pinned purpose: Get the state for a routing control. A routing control is a simple on/off switch that you can use to route traffic to cells. When a routing control state is set to ON, traffic flows to a cell. When the state is set to OFF, traffic does not flow. Befo")),
    ("ListRoutingControls", Decision::Deferred("Data-plane API requires a cluster endpoint from recovery-control-config DescribeCluster.ClusterEndpoints rather than the fixed regional endpoint. The current route catalog has no parent-derived endpoint injection; the configuration API ListRoutingControls is the safe root.")),
    // Changes service state rather than reading inventory. Pinned purpose: Set the state of the routing control to reroute traffic. You can set the value to ON or OFF. When the state is ON, traffic flows to a cell. When the state is OFF, traffic does not flow. With Route 53 ARC, you can add safety rules for routing controls, which are safeguards for routing control state updates that help prevent unexpected outcomes, like fail open traffic routing. However, there are sce
    ("UpdateRoutingControlState", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Set multiple routing control states. You can set the value for each state to be ON or OFF. When the state is ON, traffic flows to a cell. When it's OFF, traffic does not flow. With Route 53 ARC, you can add safety rules for routing controls, which are safeguards for routing control state updates that help prevent unexpected outcomes, like fail open traffic routing. However, there are scenarios whe
    ("UpdateRoutingControlStates", Decision::Denied(DenyReason::Mutating)),
];
