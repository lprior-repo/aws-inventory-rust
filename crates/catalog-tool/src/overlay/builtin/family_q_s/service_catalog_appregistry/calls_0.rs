// Pinned model: .local/aws-models/models/service-catalog-appregistry/service/2020-06-24/service-catalog-appregistry-2020-06-24.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListApplications",
        items: "applications",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListAttributeGroups",
        items: "attributeGroups",
        identity: ResourceIdentity::Arn("arn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
