use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AcceptResourceShareInvitation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateResourceShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateResourceSharePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreatePermission", Decision::Denied(DenyReason::Mutating)),
    (
        "CreatePermissionVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateResourceShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeletePermission", Decision::Denied(DenyReason::Mutating)),
    (
        "DeletePermissionVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResourceShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResourceShare",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResourceSharePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableSharingWithAwsOrganization",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetPermission",
        Decision::Deferred(
            "Required selectors permissionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members permission. Pinned purpose: Retrieves the contents of a managed permission in JSON format. If the permission has template parameters, you can include the permissionTemplateParameters parameter in your request. The operation then renders the returned policy with the specified va",
        ),
    ),
    (
        "GetResourcePolicies",
        Decision::Deferred(
            "Required selectors resourceArns are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members policies, nextToken. Pinned purpose: Retrieves the resource policies for the specified resources that you own and have shared. Always check the NextToken response parameter for a null value when calling a paginated operation. These operations can occasionally return an empty set of resu",
        ),
    ),
    (
        "GetResourceShareAssociations",
        Decision::Deferred(
            "Required selectors associationType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourceShareAssociations, nextToken. Pinned purpose: Retrieves the lists of resources and principals that associated for resource shares that you own. Always check the NextToken response parameter for a null value when calling a paginated operation. These operations can occasionally return an empty set",
        ),
    ),
    (
        "ListPendingInvitationResources",
        Decision::Deferred(
            "Required selectors resourceShareInvitationArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resources, nextToken. Pinned purpose: Lists the resources in a resource share that is shared with you but for which the invitation is still PENDING . That means that you haven't accepted or rejected the invitation and the invitation hasn't expired. Always check the NextToken response par",
        ),
    ),
    (
        "ListPermissionVersions",
        Decision::Deferred(
            "Required selectors permissionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members permissions, nextToken. Pinned purpose: Lists the available versions of the specified RAM permission. Always check the NextToken response parameter for a null value when calling a paginated operation. These operations can occasionally return an empty set of results even when there are more",
        ),
    ),
    (
        "ListPrincipals",
        Decision::Deferred(
            "Required selectors resourceOwner are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members principals, nextToken. Pinned purpose: Lists the principals that you are sharing resources with or that are sharing resources with you. Always check the NextToken response parameter for a null value when calling a paginated operation. These operations can occasionally return an empty set ",
        ),
    ),
    (
        "ListResourceSharePermissions",
        Decision::Deferred(
            "Required selectors resourceShareArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members permissions, nextToken. Pinned purpose: Lists the RAM permissions that are associated with a resource share. Always check the NextToken response parameter for a null value when calling a paginated operation. These operations can occasionally return an empty set of results even when there a",
        ),
    ),
    (
        "ListResourceTypes",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListResources",
        Decision::Deferred(
            "Required selectors resourceOwner are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resources, nextToken. Pinned purpose: Lists the resources that you added to a resource share or the resources that are shared with you. Always check the NextToken response parameter for a null value when calling a paginated operation. These operations can occasionally return an empty set",
        ),
    ),
    (
        "PromotePermissionCreatedFromPolicy",
        Decision::Deferred(
            "Required selectors permissionArn, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members permission, clientToken. Pinned purpose: When you attach a resource-based policy to a resource, RAM automatically creates a resource share of featureSet = CREATED_FROM_POLICY with a managed permission that has the same IAM permissions as the original resource-based policy. However, this typ",
        ),
    ),
    (
        "PromoteResourceShareCreatedFromPolicy",
        Decision::Deferred(
            "Required selectors resourceShareArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members returnValue. Pinned purpose: When you attach a resource-based policy to a resource, RAM automatically creates a resource share of featureSet = CREATED_FROM_POLICY with a managed permission that has the same IAM permissions as the original resource-based policy. However, this typ",
        ),
    ),
    (
        "RejectResourceShareInvitation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ReplacePermissionAssociations",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetDefaultPermissionVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdatePermission", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateResourceShare",
        Decision::Denied(DenyReason::Mutating),
    ),
];
