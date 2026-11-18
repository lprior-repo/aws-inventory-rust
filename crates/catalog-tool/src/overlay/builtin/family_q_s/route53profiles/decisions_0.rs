// Pinned operation purposes: .local/aws-models/models/route53profiles/service/2018-05-10/route53profiles-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Associates a Route 53 Profiles profile with a VPC. A VPC can have up to two Profile associations. A Profile can be associated with up to 1,000 VPCs, and you can request a higher quota for the number of VPCs. For more information, see Quotas on Route 53 Profiles . Route 53 VPC Resolver evaluates the Profile associations for a VPC in priority order, starting with the lowest Priority value. If the VP
    ("AssociateProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates a DNS resource configuration to a Route 53 Profile.
    (
        "AssociateResourceToProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an empty Route 53 Profile.
    ("CreateProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified Route 53 Profile. Before you can delete a profile, you must first disassociate it from all VPCs.
    ("DeleteProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Dissociates a specified Route 53 Profile from the specified VPC.
    (
        "DisassociateProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Dissoaciated a specified resource, from the Route 53 Profile.
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds one or more tags to a specified resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes one or more tags from a specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the name or priority of the specified Route 53 Profile association with a VPC. Route 53 VPC Resolver evaluates the Profile associations for a VPC in priority order, starting with the lowest value. Only the account that created the association can update it. When you change the priority, the association status is UPDATING until the change propagates, and then COMPLETE . While the status is
    (
        "UpdateProfileAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified Route 53 Profile resourse association.
    (
        "UpdateProfileResourceAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
];
