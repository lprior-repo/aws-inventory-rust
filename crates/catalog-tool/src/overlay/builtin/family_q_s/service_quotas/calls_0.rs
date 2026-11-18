// Pinned model: .local/aws-models/models/service-quotas/service/2019-06-24/service-quotas-2019-06-24.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListRequestedServiceQuotaChangeHistory",
    items: "RequestedQuotas",
    identity: ResourceIdentity::Key(&["Id"]),
    token: Some(("NextToken", "NextToken")),
    scope: Scope::Region,
    input: &[],
    tags: None,
    queryable: &[],
}];
