// Pinned operation purposes: .local/aws-models/models/resource-groups/service/2017-11-27/resource-groups-2017-11-27.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Cancels the specified tag-sync task. Minimum permissions To run this command, you must have the following permissions: resource-groups:CancelTagSyncTask on the application group resource-groups:DeleteGroup
    ("CancelTagSyncTask", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a resource group with the specified name and description. You can optionally include either a resource query or a service configuration. For more information about constructing a resource query, see Build queries and groups in Resource Groups in the Resource Groups User Guide . For more information about service-linked groups and service configurations, see Service configurations for Resou
    ("CreateGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified resource group. Deleting a resource group does not delete any resources that are members of the group; it only deletes the group structure. Minimum permissions To run this command, you must have the following permissions: resource-groups:DeleteGroup
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds the specified resources to the specified group. You can only use this operation with the following groups: AWS::EC2::HostManagement AWS::EC2::CapacityReservationPool AWS::ResourceGroups::ApplicationGroup Other resource group types and resource types are not currently supported by this operation. Minimum permissions To run this command, you must have the following permissions: resource-groups:
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
    // Changes service state rather than reading inventory. Pinned purpose: Attaches a service configuration to the specified group. This occurs asynchronously, and can take time to complete. You can use GetGroupConfiguration to check the status of the update. Minimum permissions To run this command, you must have the following permissions: resource-groups:PutGroupConfiguration
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
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new tag-sync task to onboard and sync resources tagged with a specific tag key-value pair to an application. To start a tag-sync task, you need a resource tagging role . The resource tagging role grants permissions to tag and untag applications resources and must include a trust policy that allows Resource Groups to assume the role and perform resource tagging tasks on your behalf. For i
    ("StartTagSyncTask", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds tags to a resource group with the specified Amazon resource name (ARN). Existing tags on a resource group are not changed if they are not specified in the request parameters. Do not store personally identifiable information (PII) or other confidential or sensitive information in tags. We use tags to provide you with billing and administration services. Tags are not intended to be used for pri
    ("Tag", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified resources from the specified group. This operation works only with static groups that you populated using the GroupResources operation. It doesn't work with any resource groups that are automatically populated by tag-based or CloudFormation stack-based queries. Minimum permissions To run this command, you must have the following permissions: resource-groups:UngroupResources
    ("UngroupResources", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes tags from a specified resource group. Minimum permissions To run this command, you must have the following permissions: resource-groups:Untag
    ("Untag", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Turns on or turns off optional features in Resource Groups. The preceding example shows that the request to turn on group lifecycle events is IN_PROGRESS . You can call the GetAccountSettings operation to check for completion by looking for GroupLifecycleEventsStatus to change to ACTIVE .
    (
        "UpdateAccountSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the description for an existing group. You cannot update the name of a resource group. Minimum permissions To run this command, you must have the following permissions: resource-groups:UpdateGroup
    ("UpdateGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource query of a group. For more information about resource queries, see Create a tag-based group in Resource Groups . Minimum permissions To run this command, you must have the following permissions: resource-groups:UpdateGroupQuery
    ("UpdateGroupQuery", Decision::Denied(DenyReason::Mutating)),
];
