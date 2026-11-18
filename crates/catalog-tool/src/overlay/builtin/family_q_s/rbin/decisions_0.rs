// Pinned operation purposes: .local/aws-models/models/rbin/service/2021-06-15/rbin-2021-06-15.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Recycle Bin retention rule. You can create two types of retention rules: Tag-level retention rules - These retention rules use resource tags to identify the resources to protect. For each retention rule, you specify one or more tag key and value pairs. Resources (of the specified type) that have at least one of these tag key and value pairs are automatically retained in the Recycle Bin u
    ("CreateRule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Recycle Bin retention rule. For more information, see Delete Recycle Bin retention rules in the Amazon Elastic Compute Cloud User Guide .
    ("DeleteRule", Decision::Denied(DenyReason::Mutating)),
    (
        "GetRule",
        Decision::Deferred(
            "Required selectors Identifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Identifier, Description, ResourceType, RetentionPeriod, ResourceTags, Status, LockConfiguration, LockState, LockEndTime, RuleArn, ExcludeResourceTags. Pinned purpose: Gets information about a Recycle Bin retention rule.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags assigned to a retention rule.",
        ),
    ),
    (
        "LockRule",
        Decision::Deferred(
            "Required selectors Identifier, LockConfiguration are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Identifier, Description, ResourceType, RetentionPeriod, ResourceTags, Status, LockConfiguration, LockState, RuleArn, ExcludeResourceTags. Pinned purpose: Locks a Region-level retention rule. A locked retention rule can't be modified or deleted. You can't lock tag-level retention rules, or Region-level retention rules that have exclusion tags.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns tags to the specified retention rule.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UnlockRule",
        Decision::Deferred(
            "Required selectors Identifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Identifier, Description, ResourceType, RetentionPeriod, ResourceTags, Status, LockConfiguration, LockState, LockEndTime, RuleArn, ExcludeResourceTags. Pinned purpose: Unlocks a retention rule. After a retention rule is unlocked, it can be modified or deleted only after the unlock delay period expires.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Unassigns a tag from a retention rule.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Recycle Bin retention rule. You can update a retention rule's description, resource tags, and retention period at any time after creation. You can't update a retention rule's resource type after creation. For more information, see Update Recycle Bin retention rules in the Amazon Elastic Compute Cloud User Guide .
    ("UpdateRule", Decision::Denied(DenyReason::Mutating)),
];
