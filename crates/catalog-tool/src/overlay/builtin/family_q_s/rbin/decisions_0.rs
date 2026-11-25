use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateRule", Decision::Denied(DenyReason::Mutating)),
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
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UnlockRule",
        Decision::Deferred(
            "Required selectors Identifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Identifier, Description, ResourceType, RetentionPeriod, ResourceTags, Status, LockConfiguration, LockState, LockEndTime, RuleArn, ExcludeResourceTags. Pinned purpose: Unlocks a retention rule. After a retention rule is unlocked, it can be modified or deleted only after the unlock delay period expires.",
        ),
    ),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateRule", Decision::Denied(DenyReason::Mutating)),
];
