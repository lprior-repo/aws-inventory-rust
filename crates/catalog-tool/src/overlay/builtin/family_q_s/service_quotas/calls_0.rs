use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[Inventory {
    operation: "ListRequestedServiceQuotaChangeHistory",
    items: "RequestedQuotas",
    identity: ResourceIdentity::Key(&["Id"]),
    token: Some(("NextToken", "NextToken")),
    scope: Scope::Region,
    input: &[Input::String("QuotaRequestedAtLevel", "ALL")],
    tags: None,
    queryable: &[],
}];
