use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateDeployment",
            "CreateDeploymentSnapshot",
            "CreatePolicy",
            "CreatePolicySnapshot",
            "CreateRule",
            "CreateRuleSnapshot",
            "CreateScope",
            "CreateScopeSnapshot",
            "CreateTemplate",
            "CreateTemplateSnapshot",
            "DeleteAdminAccount",
            "DeleteDeployment",
            "DeletePolicy",
            "DeleteRule",
            "DeleteScope",
            "DeleteTemplate",
            "GenerateRuleConfiguration",
            "PutAdminAccount",
            "TagResource",
            "UntagResource",
            "UpdateDeployment",
            "UpdatePolicy",
            "UpdateRule",
            "UpdateScope",
            "UpdateTemplate",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetAdminAccount"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (accountId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetDeployment",
            "ListDeploymentSnapshots",
            "ListResourceSynchronizationStatuses",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (deploymentIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetPolicy", "ListPolicySnapshots"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (policyIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRule", "ListRuleSnapshots"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ruleIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetScope", "ListScopeSnapshots"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (scopeIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplate", "ListTemplateSnapshots"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListAdminAccounts",
            "ListAggregateResourceSynchronizationStatuses",
            "ListTemplates",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListResourceAssociations"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListScopes",
        items: "scopes",
        id: Id::Arn("scopeArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["scopeArn", "scopeId", "scopeName", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListPolicies",
        items: "policies",
        id: Id::Arn("policyArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "policyArn",
            "policyId",
            "policyName",
            "status",
            "firewallType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListRules",
        items: "rules",
        id: Id::Arn("ruleArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "ruleArn",
            "ruleId",
            "ruleName",
            "firewallType",
            "ruleType",
            "status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListDeployments",
        items: "deployments",
        id: Id::Arn("deploymentArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["deploymentArn", "deploymentId", "deploymentName", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("network-security-manager", GROUPS, RESOURCES)
}
