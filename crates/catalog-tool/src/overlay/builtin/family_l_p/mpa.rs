// Pinned Smithy source: .local/aws-models/models/mpa/service/2022-07-26/mpa-2022-07-26.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CancelSession", "CreateApprovalTeam", "CreateIdentitySource", 
            "DeleteIdentitySource", "DeleteInactiveApprovalTeamVersion", 
            "StartActiveApprovalTeamDeletion", "StartApprovalTeamBaseline", "TagResource", 
            "UntagResource", "UpdateApprovalTeam",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetApprovalTeam",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Arn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetIdentitySource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (IdentitySourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetPolicyVersion",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PolicyVersionArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetResourcePolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn, PolicyName, PolicyType); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetSession",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (SessionArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListPolicies",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListPolicyVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (PolicyArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListResourcePolicies", "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListSessions",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ApprovalTeamArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a list of approval teams.
    Resource {
        operation: "ListApprovalTeams",
        items: "ApprovalTeams", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of identity sources. For more information, see Identity Source in the Multi-party approval User Guide .
    Resource {
        operation: "ListIdentitySources",
        items: "IdentitySources", id: Id::Arn("IdentitySourceArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["IdentitySourceArn", "IdentitySourceType", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mpa", GROUPS, RESOURCES)
}
