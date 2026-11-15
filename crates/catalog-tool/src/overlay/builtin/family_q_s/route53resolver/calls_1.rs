// Pinned model: .local/aws-models/models/route53resolver/service/2018-04-01/route53resolver-2018-04-01.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListResolverConfigs",
        items: "ResolverConfigs",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListResolverDnssecConfigs",
        items: "ResolverDnssecConfigs",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
