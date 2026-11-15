// Pinned Smithy source: .local/aws-models/models/managedblockchain/service/2018-09-24/managedblockchain-2018-09-24.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateAccessor", "CreateMember", "CreateNetwork", "CreateNode", "CreateProposal", 
            "DeleteAccessor", "DeleteMember", "DeleteNode", "RejectInvitation", "TagResource", 
            "UntagResource", "UpdateMember", "UpdateNode", "VoteOnProposal",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAccessor",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &[
            "GetMember",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (NetworkId, MemberId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetNetwork", "ListNodes", "ListProposals",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (NetworkId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetNode",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (NetworkId, NodeId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetProposal", "ListProposalVotes",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (NetworkId, ProposalId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListInvitations",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a list of the accessors and their properties. Accessor objects are containers that have the information required for token based access to your Ethereum nodes.
    Resource {
        operation: "ListAccessors",
        items: "Accessors", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Type", "Status", "NetworkType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns information about the networks in which the current Amazon Web Services account participates. Applies to Hyperledger Fabric and Ethereum.
    Resource {
        operation: "ListNetworks",
        items: "Networks", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of the members in a network and properties of their configurations. Applies only to Hyperledger Fabric.
    Resource {
        operation: "ListMembers",
        items: "Members", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "Status"],
        tags: None,
        bindings: &[("NetworkId", "ListNetworks", "Id")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("managedblockchain", GROUPS, RESOURCES)
}
