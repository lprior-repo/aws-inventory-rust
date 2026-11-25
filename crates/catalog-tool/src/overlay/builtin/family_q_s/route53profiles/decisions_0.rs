use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AssociateProfile", Decision::Denied(DenyReason::Mutating)),
    (
        "AssociateResourceToProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateProfile", Decision::Denied(DenyReason::Mutating)),
    ("DeleteProfile", Decision::Denied(DenyReason::Mutating)),
    (
        "DisassociateProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResourceFromProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetProfile",
        Decision::Deferred(
            "Required selectors ProfileId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Profile. Pinned purpose: Returns information about a specified Route 53 Profile, such as whether whether the Profile is shared, and the current status of the Profile.",
        ),
    ),
    (
        "GetProfileAssociation",
        Decision::Deferred(
            "Required selectors ProfileAssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProfileAssociation. Pinned purpose: Retrieves a Route 53 Profile association for a VPC. A VPC can have up to two Profile associations, and a Profile can be associated with up to 1,000 VPCs.",
        ),
    ),
    (
        "GetProfileResourceAssociation",
        Decision::Deferred(
            "Required selectors ProfileResourceAssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProfileResourceAssociation. Pinned purpose: Returns information about a specified Route 53 Profile resource association.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags that you associated with the specified resource.",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateProfileAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateProfileResourceAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
];
