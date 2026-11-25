use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListConfigurationSets",
        items: "ConfigurationSets",
        identity: ResourceIdentity::Template(
            "arn:{partition}:ses:{region}:{account}:configuration-set/{Name}",
            &["Name"],
        ),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListIdentities",
        items: "Identities",
        identity: ResourceIdentity::Template(
            "arn:{partition}:ses:{region}:{account}:identity/{$}",
            &["$"],
        ),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListReceiptRuleSets",
        items: "RuleSets",
        identity: ResourceIdentity::Key(&["Name"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListTemplates",
        items: "TemplatesMetadata",
        identity: ResourceIdentity::Template(
            "arn:{partition}:ses:{region}:{account}:template/{Name}",
            &["Name"],
        ),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListReceiptFilters",
        items: "Filters",
        identity: ResourceIdentity::Key(&["Name"]),
        token: None,
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
];
