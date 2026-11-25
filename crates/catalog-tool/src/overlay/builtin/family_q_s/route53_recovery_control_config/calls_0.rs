use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListClusters",
        items: "Clusters",
        identity: ResourceIdentity::Arn("ClusterArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListControlPanels",
        items: "ControlPanels",
        identity: ResourceIdentity::Arn("ControlPanelArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListRoutingControls",
        items: "RoutingControls",
        identity: ResourceIdentity::Arn("RoutingControlArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Account,
        input: &[Input::Parent(
            "ControlPanelArn",
            "ListControlPanels",
            "ControlPanelArn",
        )],
        tags: None,
        queryable: &[],
    },
];
