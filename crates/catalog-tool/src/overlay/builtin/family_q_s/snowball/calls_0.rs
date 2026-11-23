// Pinned model: .local/aws-models/models/snowball/service/2016-06-30/snowball-2016-06-30.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListClusters",
        items: "ClusterListEntries",
        identity: ResourceIdentity::Key(&["ClusterId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListLongTermPricing",
        items: "LongTermPricingEntries",
        identity: ResourceIdentity::Key(&["LongTermPricingId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
