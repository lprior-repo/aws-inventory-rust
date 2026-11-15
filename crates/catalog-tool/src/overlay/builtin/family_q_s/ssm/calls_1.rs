// Pinned model: .local/aws-models/models/ssm/service/2014-11-06/ssm-2014-11-06.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "DescribePatchGroups",
        items: "Mappings",
        identity: ResourceIdentity::Key(&["PatchGroup"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListComplianceSummaries",
        items: "ComplianceSummaryItems",
        identity: ResourceIdentity::Key(&["ComplianceType"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListOpsItemRelatedItems",
        items: "Summaries",
        identity: ResourceIdentity::Key(&["OpsItemId", "AssociationId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListResourceComplianceSummaries",
        items: "ResourceComplianceSummaryItems",
        identity: ResourceIdentity::Key(&["ComplianceType", "ResourceType", "ResourceId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
