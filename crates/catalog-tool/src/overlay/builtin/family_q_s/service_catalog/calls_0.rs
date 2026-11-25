use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListPortfolios",
        items: "PortfolioDetails",
        identity: ResourceIdentity::Arn("ARN"),
        token: Some(("PageToken", "NextPageToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListTagOptions",
        items: "TagOptionDetails",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("PageToken", "PageToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListServiceActions",
        items: "ServiceActionSummaries",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("PageToken", "NextPageToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListProvisionedProductPlans",
        items: "ProvisionedProductPlans",
        identity: ResourceIdentity::Key(&["PlanId"]),
        token: Some(("PageToken", "NextPageToken")),
        scope: Scope::Region,
        input: &[Input::Structure(
            "AccessLevelFilter",
            &[("Key", "Account"), ("Value", "self")],
        )],
        tags: None,
        queryable: &[],
    },
];
