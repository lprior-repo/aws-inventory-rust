use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateDRTLogBucket",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("AssociateDRTRole", Decision::Denied(DenyReason::Mutating)),
    (
        "AssociateHealthCheck",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateProactiveEngagementDetails",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateProtection", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateProtectionGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSubscription", Decision::Denied(DenyReason::Mutating)),
    ("DeleteProtection", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteProtectionGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSubscription", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeAttack",
        Decision::Deferred(
            "Required selectors AttackId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Attack. Pinned purpose: Describes the details of a DDoS attack.",
        ),
    ),
    (
        "DescribeAttackStatistics",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members TimeRange, DataItems. Pinned request/ownership contract: Provides information about the number and type of attacks Shield has detected in the last year for all resources that belong to your account, regardless of whether you've defined Shield protections for them. This operation is available to Shield customers as well as to Shield Advanced customers. The operation returns data for the time range of midnight UTC, one year ago, to midnight UTC, today. For example, if the current time is 2020-10-26 15:39",
        ),
    ),
    (
        "DescribeDRTAccess",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members RoleArn, LogBucketList. Pinned request/ownership contract: Returns the current role and list of Amazon S3 log buckets used by the Shield Response Team (SRT) to access your Amazon Web Services account while assisting with attack mitigation.",
        ),
    ),
    (
        "DescribeEmergencyContactSettings",
        Decision::Deferred(
            "Contains human emergency contact email/phone records, not resource configuration identity.",
        ),
    ),
    (
        "DescribeProtection",
        Decision::Deferred(
            "Requires either ProtectionId or ResourceArn although both are optional in the pinned structure. ListProtections supplies the complete protection root.",
        ),
    ),
    (
        "DescribeProtectionGroup",
        Decision::Deferred(
            "Required selectors ProtectionGroupId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProtectionGroup. Pinned purpose: Returns the specification for the specified protection group.",
        ),
    ),
    (
        "DescribeSubscription",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Subscription. Pinned request/ownership contract: Provides details about the Shield Advanced subscription for an account.",
        ),
    ),
    (
        "DisableApplicationLayerAutomaticResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisableProactiveEngagement",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateDRTLogBucket",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateDRTRole",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateHealthCheck",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableApplicationLayerAutomaticResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableProactiveEngagement",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetSubscriptionState",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members SubscriptionState. Pinned request/ownership contract: Returns the SubscriptionState , either Active or Inactive .",
        ),
    ),
    (
        "ListResourcesInProtectionGroup",
        Decision::Deferred(
            "Required selectors ProtectionGroupId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceArns, NextToken. Pinned purpose: Retrieves the resources that are included in the protection group.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceARN. Pinned purpose: Gets information about Amazon Web Services tags for a specified Amazon Resource Name (ARN) in Shield.",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateApplicationLayerAutomaticResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateEmergencyContactSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateProtectionGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSubscription", Decision::Denied(DenyReason::Mutating)),
];
