use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "GetRoutingControlState",
        Decision::Deferred(
            "Required selectors RoutingControlArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RoutingControlArn, RoutingControlState, RoutingControlName. Pinned purpose: Get the state for a routing control. A routing control is a simple on/off switch that you can use to route traffic to cells. When a routing control state is set to ON, traffic flows to a cell. When the state is set to OFF, traffic does not flow. Befo",
        ),
    ),
    (
        "ListRoutingControls",
        Decision::Deferred(
            "Data-plane API requires a cluster endpoint from recovery-control-config DescribeCluster.ClusterEndpoints rather than the fixed regional endpoint. The current route catalog has no parent-derived endpoint injection; the configuration API ListRoutingControls is the safe root.",
        ),
    ),
    (
        "UpdateRoutingControlState",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateRoutingControlStates",
        Decision::Denied(DenyReason::Mutating),
    ),
];
