use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListPageReceipts",
        Decision::Deferred(
            "receipt identity requires ReceiptTime; timestamp identity representation is unsupported",
        ),
    ),
    (
        "ListRotationShifts",
        Decision::Deferred(
            "Computed shifts over a supplied time range are not stored inventory; the shift type alone does not identify a shift.",
        ),
    ),
    ("AcceptPage", Decision::Denied(DenyReason::Mutating)),
    (
        "ActivateContactChannel",
        Decision::Deferred(
            "Required selectors ContactChannelId, ActivationCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Activates a contact's contact channel. Incident Manager can't engage a contact until the contact channel has been activated.",
        ),
    ),
    ("CreateContact", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateContactChannel",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateRotation", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateRotationOverride",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeactivateContactChannel",
        Decision::Deferred(
            "Required selectors ContactChannelId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: To no longer receive Incident Manager engagements to a contact channel, you can deactivate the channel.",
        ),
    ),
    ("DeleteContact", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteContactChannel",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteRotation", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteRotationOverride",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeEngagement",
        Decision::Deferred(
            "Required selectors EngagementId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ContactArn, EngagementArn, Sender, Subject, Content, PublicSubject, PublicContent, IncidentId, StartTime, StopTime. Pinned purpose: Incident Manager uses engagements to engage contacts and escalation plans during an incident. Use this command to describe the engagement that occurred during an incident.",
        ),
    ),
    (
        "DescribePage",
        Decision::Deferred(
            "Required selectors PageId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PageArn, EngagementArn, ContactArn, Sender, Subject, Content, PublicSubject, PublicContent, IncidentId, SentTime, ReadTime, DeliveryTime. Pinned purpose: Lists details of the engagement to a contact channel.",
        ),
    ),
    (
        "GetContact",
        Decision::Deferred(
            "Required selectors ContactId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ContactArn, Alias, DisplayName, Type, Plan. Pinned purpose: Retrieves information about the specified contact or escalation plan.",
        ),
    ),
    (
        "GetContactChannel",
        Decision::Deferred(
            "Required selectors ContactChannelId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ContactArn, ContactChannelArn, Name, Type, DeliveryAddress, ActivationStatus. Pinned purpose: List details about a specific contact channel.",
        ),
    ),
    (
        "GetContactPolicy",
        Decision::Deferred(
            "Required selectors ContactArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ContactArn, Policy. Pinned purpose: Retrieves the resource policies attached to the specified contact or escalation plan.",
        ),
    ),
    (
        "GetRotation",
        Decision::Deferred(
            "Required selectors RotationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RotationArn, Name, ContactIds, StartTime, TimeZoneId, Recurrence. Pinned purpose: Retrieves information about an on-call rotation.",
        ),
    ),
    (
        "GetRotationOverride",
        Decision::Deferred(
            "Required selectors RotationId, RotationOverrideId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RotationOverrideId, RotationArn, NewContactIds, StartTime, EndTime, CreateTime. Pinned purpose: Retrieves information about an override to an on-call rotation.",
        ),
    ),
    (
        "ListPreviewRotationShifts",
        Decision::Deferred(
            "Required selectors EndTime, Members, TimeZoneId, Recurrence are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RotationShifts, NextToken. Pinned purpose: Returns a list of shifts based on rotation configuration parameters. The Incident Manager primarily uses this operation to populate the Preview calendar. It is not typically run by end users.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceARN. Pinned purpose: Lists the tags of a contact, escalation plan, rotation, or on-call schedule.",
        ),
    ),
    ("PutContactPolicy", Decision::Denied(DenyReason::Mutating)),
    ("SendActivationCode", Decision::Denied(DenyReason::Mutating)),
    ("StartEngagement", Decision::Denied(DenyReason::Mutating)),
    ("StopEngagement", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateContact", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateContactChannel",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateRotation", Decision::Denied(DenyReason::Mutating)),
];
