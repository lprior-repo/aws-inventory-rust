use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListPermissionSetProvisioningStatus",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PermissionSetsProvisioningStatus, NextToken. Pinned purpose: Lists the status of the permission set provisioning requests for a specified IAM Identity Center instance.",
        ),
    ),
    (
        "ListPermissionSetsProvisionedToAccount",
        Decision::Deferred(
            "Required selectors InstanceArn, AccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, PermissionSets. Pinned purpose: Lists all the permission sets that are provisioned to a specified Amazon Web Services account.",
        ),
    ),
    (
        "ListRegions",
        Decision::Deferred(
            "Required selectors InstanceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Regions, NextToken. Pinned purpose: Lists all enabled Regions of an IAM Identity Center instance, including those that are being added or removed. This operation returns Regions with ACTIVE, ADDING, or REMOVING status. The following actions are related to ListRegions : AddRegion Remove",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags that are attached to a specified resource.",
        ),
    ),
    (
        "ProvisionPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutApplicationAccessScope",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutApplicationAssignmentConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutApplicationAuthenticationMethod",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutApplicationGrant",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutApplicationSessionConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutInlinePolicyToPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutPermissionsBoundaryToPermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("RemoveRegion", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    ("UpdateInstance", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateInstanceAccessControlAttributeConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdatePermissionSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateTrustedTokenIssuer",
        Decision::Denied(DenyReason::Mutating),
    ),
];
