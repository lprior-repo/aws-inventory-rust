use super::policy::{Group, Id, Input, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateAccessor",
            "CreateMember",
            "CreateNetwork",
            "CreateNode",
            "CreateProposal",
            "DeleteAccessor",
            "DeleteMember",
            "DeleteNode",
            "RejectInvitation",
            "TagResource",
            "UntagResource",
            "UpdateMember",
            "UpdateNode",
            "VoteOnProposal",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetAccessor"],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["GetMember"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (NetworkId, MemberId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetNetwork", "ListNodes", "ListProposals"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (NetworkId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetNode"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (NetworkId, NodeId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetProposal", "ListProposalVotes"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (NetworkId, ProposalId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListInvitations"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListAccessors",
        items: "Accessors",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Type", "Status", "NetworkType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListNetworks",
        items: "Networks",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListMembers",
        items: "Members",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "Status"],
        tags: None,
        bindings: &[("NetworkId", "ListNetworks", "Id")],
        fixed: &[Input::Boolean("IsOwned", true)],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("managedblockchain", GROUPS, RESOURCES)
}
