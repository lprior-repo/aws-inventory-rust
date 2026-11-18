// Pinned model: .local/aws-models/models/route53profiles/service/2018-05-10/route53profiles-2018-05-10.json
use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListProfiles",
        items: "ProfileSummaries",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &["Id"],
    },
    Inventory {
        operation: "ListProfileAssociations",
        items: "ProfileAssociations",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &["ProfileId", "ResourceId"],
    },
    Inventory {
        operation: "ListProfileResourceAssociations",
        items: "ProfileResourceAssociations",
        identity: ResourceIdentity::Key(&["Id"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[Input::Parent("ProfileId", "ListProfiles", "Id")],
        tags: None,
        queryable: &[],
    },
];
