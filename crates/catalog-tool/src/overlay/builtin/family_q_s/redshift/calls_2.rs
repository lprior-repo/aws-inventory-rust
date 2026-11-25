use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "DescribeTableRestoreStatus",
        items: "TableRestoreStatusDetails",
        identity: ResourceIdentity::Key(&["TableRestoreRequestId"]),
        token: Some(("Marker", "Marker")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "DescribeClusterDbRevisions",
        items: "ClusterDbRevisions",
        identity: ResourceIdentity::Key(&["ClusterIdentifier"]),
        token: Some(("Marker", "Marker")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListRecommendations",
        items: "Recommendations",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("Marker", "Marker")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
