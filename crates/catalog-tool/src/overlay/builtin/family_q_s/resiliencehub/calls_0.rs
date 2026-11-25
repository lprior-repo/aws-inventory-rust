use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListApps",
        items: "appSummaries",
        identity: ResourceIdentity::Arn("appArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListResiliencyPolicies",
        items: "resiliencyPolicies",
        identity: ResourceIdentity::Arn("policyArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListAppAssessments",
        items: "assessmentSummaries",
        identity: ResourceIdentity::Arn("assessmentArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListRecommendationTemplates",
        items: "recommendationTemplates",
        identity: ResourceIdentity::Arn("recommendationTemplateArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("tags"),
        queryable: &[],
    },
];
