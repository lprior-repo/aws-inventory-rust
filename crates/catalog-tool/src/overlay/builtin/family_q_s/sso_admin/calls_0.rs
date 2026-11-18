// Pinned model: .local/aws-models/models/sso-admin/service/2020-07-20/sso-admin-2020-07-20.json
use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListInstances",
        items: "Instances",
        identity: ResourceIdentity::Arn("InstanceArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListPermissionSets",
        items: "PermissionSets",
        identity: ResourceIdentity::Arn("$"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[Input::Parent("InstanceArn", "ListInstances", "InstanceArn")],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListApplications",
        items: "Applications",
        identity: ResourceIdentity::Arn("ApplicationArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[
            Input::Account("Filter.ApplicationAccount"),
            Input::Parent("InstanceArn", "ListInstances", "InstanceArn"),
        ],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListTrustedTokenIssuers",
        items: "TrustedTokenIssuers",
        identity: ResourceIdentity::Arn("TrustedTokenIssuerArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[Input::Parent("InstanceArn", "ListInstances", "InstanceArn")],
        tags: None,
        queryable: &[],
    },
];
