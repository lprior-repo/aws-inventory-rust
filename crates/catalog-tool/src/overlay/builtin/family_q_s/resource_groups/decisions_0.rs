use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CancelTagSyncTask", Decision::Denied(DenyReason::Mutating)),
    ("CreateGroup", Decision::Denied(DenyReason::Mutating)),
    ("DeleteGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "GetAccountSettings",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AccountSettings. Pinned request/ownership contract: Retrieves the current status of optional features in Resource Groups.",
        ),
    ),
    (
        "GetGroup",
        Decision::Deferred(
            "Requires one of Group (ARN/name) or deprecated GroupName despite neither being a Smithy required member; unrestricted empty request is invalid. ListGroups supplies the complete root.",
        ),
    ),
    (
        "GetGroupConfiguration",
        Decision::Deferred(
            "Group selects a single resource group and is optional only in Smithy, not a complete root selector. Group-configuration detail needs a reviewed parent ARN binding.",
        ),
    ),
    (
        "GetGroupQuery",
        Decision::Deferred(
            "Returns the group query expression, not resource metadata. A Group selector is semantically required despite optional modeling.",
        ),
    ),
    (
        "GetTagSyncTask",
        Decision::Deferred(
            "Required selectors TaskArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GroupArn, GroupName, TaskArn, TagKey, TagValue, ResourceQuery, RoleArn, Status, ErrorMessage, CreatedAt. Pinned purpose: Returns information about a specified tag-sync task. Minimum permissions To run this command, you must have the following permissions: resource-groups:GetTagSyncTask on the application group",
        ),
    ),
    (
        "GetTags",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: Arn. Pinned purpose: Returns a list of tags that are associated with a resource group, specified by an Amazon resource name (ARN). Minimum permissions To run this command, you must have the following permissions: resource-groups:GetTags",
        ),
    ),
    ("GroupResources", Decision::Denied(DenyReason::Mutating)),
    (
        "ListGroupResources",
        Decision::Deferred(
            "Heterogeneous group member ARNs need dynamic canonical kind dispatch; no single static resource kind can represent the memberships without an explicit parent-group association identity.",
        ),
    ),
    (
        "ListGroupingStatuses",
        Decision::Deferred(
            "Required selectors Group are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Group, GroupingStatuses, NextToken. Pinned purpose: Returns the status of the last grouping or ungrouping action for each resource in the specified application group.",
        ),
    ),
    (
        "PutGroupConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SearchResources",
        Decision::Deferred(
            "Required selectors ResourceQuery are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceIdentifiers, NextToken, QueryErrors. Pinned purpose: Returns a list of Amazon Web Services resource identifiers that matches the specified query. The query uses the same format as a resource query in a CreateGroup or UpdateGroupQuery operation. Minimum permissions To run this command, you must have the",
        ),
    ),
    ("StartTagSyncTask", Decision::Denied(DenyReason::Mutating)),
    ("Tag", Decision::Denied(DenyReason::Mutating)),
    ("UngroupResources", Decision::Denied(DenyReason::Mutating)),
    ("Untag", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAccountSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateGroup", Decision::Denied(DenyReason::Mutating)),
    ("UpdateGroupQuery", Decision::Denied(DenyReason::Mutating)),
];
