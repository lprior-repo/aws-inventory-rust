use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateAwsLogSource", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateCustomLogSource",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateDataLake", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateDataLakeExceptionSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateDataLakeOrganizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSubscriber", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateSubscriberNotification",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAwsLogSource", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteCustomLogSource",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteDataLake", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteDataLakeExceptionSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteDataLakeOrganizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSubscriber", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteSubscriberNotification",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeregisterDataLakeDelegatedAdministrator",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetDataLakeExceptionSubscription",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members subscriptionProtocol, notificationEndpoint, exceptionTimeToLive. Pinned request/ownership contract: Retrieves the protocol and endpoint that were provided when subscribing to Amazon SNS topics for exception notifications.",
        ),
    ),
    (
        "GetDataLakeOrganizationConfiguration",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members autoEnableNewAccount. Pinned request/ownership contract: Retrieves the configuration that will be automatically set up for accounts added to the organization after the organization has onboarded to Amazon Security Lake. This API does not take input parameters.",
        ),
    ),
    (
        "GetDataLakeSources",
        Decision::Deferred(
            "Returns per-source accounts and Regions; this is not a locally scoped source resource root. Requires per-item account/Region association scope.",
        ),
    ),
    (
        "GetSubscriber",
        Decision::Deferred(
            "Required selectors subscriberId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members subscriber. Pinned purpose: Retrieves the subscription information for the specified subscription ID. You can get information about a specific subscriber.",
        ),
    ),
    (
        "ListDataLakeExceptions",
        Decision::Deferred(
            "Operational exception records have no stable resource ARN and can cover multiple Regions; resource identity and per-item routing cannot be inferred from the message.",
        ),
    ),
    (
        "ListLogSources",
        Decision::Deferred(
            "Lists supported/selected log-source capabilities across accounts and Regions. A fixed local resource scope would misattribute cross-account/Region summaries.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Retrieves the tags (keys and values) that are associated with an Amazon Security Lake resource: a subscriber, or the data lake configuration for your Amazon Web Services account in a particular Amazon Web Services Region.",
        ),
    ),
    (
        "RegisterDataLakeDelegatedAdministrator",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDataLake", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDataLakeExceptionSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSubscriber", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateSubscriberNotification",
        Decision::Denied(DenyReason::Mutating),
    ),
];
