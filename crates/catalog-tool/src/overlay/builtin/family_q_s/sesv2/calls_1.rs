use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListReputationEntities",
        items: "ReputationEntities",
        identity: ResourceIdentity::Key(&["ReputationEntityReference", "ReputationEntityType"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListCustomVerificationEmailTemplates",
        items: "CustomVerificationEmailTemplates",
        identity: ResourceIdentity::Key(&["TemplateName"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListDeliverabilityTestReports",
        items: "DeliverabilityTestReports",
        identity: ResourceIdentity::Key(&["ReportId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListRecommendations",
        items: "Recommendations",
        identity: ResourceIdentity::Key(&["ResourceArn", "Type"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
