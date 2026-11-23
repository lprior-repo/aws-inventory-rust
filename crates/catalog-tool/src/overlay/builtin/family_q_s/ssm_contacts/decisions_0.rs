// Pinned operation purposes: .local/aws-models/models/ssm-contacts/service/2021-05-03/ssm-contacts-2021-05-03.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListRotationShifts",
        Decision::Deferred(
            "Computed shifts over a supplied time range are not stored inventory; the shift type alone does not identify a shift.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to acknowledge an engagement to a contact channel during an incident.
    ("AcceptPage", Decision::Denied(DenyReason::Mutating)),
    (
        "ActivateContactChannel",
        Decision::Deferred(
            "Required selectors ContactChannelId, ActivationCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Activates a contact's contact channel. Incident Manager can't engage a contact until the contact channel has been activated.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Contacts are either the contacts that Incident Manager engages during an incident or the escalation plans that Incident Manager uses to engage contacts in phases during an incident.
    ("CreateContact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: A contact channel is the method that Incident Manager uses to engage your contact.
    (
        "CreateContactChannel",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a rotation in an on-call schedule.
    ("CreateRotation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an override for a rotation in an on-call schedule.
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
    // Changes service state rather than reading inventory. Pinned purpose: To remove a contact from Incident Manager, you can delete the contact. However, deleting a contact does not remove it from escalation plans and related response plans. Deleting an escalation plan also does not remove it from all related response plans. To modify an escalation plan, we recommend using the UpdateContact action to specify a different existing contact.
    ("DeleteContact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: To stop receiving engagements on a contact channel, you can delete the channel from a contact. Deleting the contact channel does not remove it from the contact's engagement plan, but the stage that includes the channel will be ignored. If you delete the only contact channel for a contact, you'll no longer be able to engage that contact during an incident.
    (
        "DeleteContactChannel",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a rotation from the system. If a rotation belongs to more than one on-call schedule, this operation deletes it from all of them.
    ("DeleteRotation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an existing override for an on-call rotation.
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds a resource policy to the specified contact or escalation plan. The resource policy is used to share the contact or escalation plan using Resource Access Manager (RAM). For more information about cross-account sharing, see Setting up cross-account functionality .
    ("PutContactPolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Sends an activation code to a contact channel. The contact can use this code to activate the contact channel in the console or with the ActivateChannel operation. Incident Manager can't engage a contact channel until it has been activated.
    ("SendActivationCode", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts an engagement to a contact or escalation plan. The engagement engages each contact specified in the incident.
    ("StartEngagement", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops an engagement before it finishes the final stage of the escalation plan or engagement plan. Further contacts aren't engaged.
    ("StopEngagement", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Tags a contact or escalation plan. You can tag only contacts and escalation plans in the first region of your replication set.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from the specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the contact or escalation plan specified.
    ("UpdateContact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a contact's contact channel.
    (
        "UpdateContactChannel",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the information specified for an on-call rotation.
    ("UpdateRotation", Decision::Denied(DenyReason::Mutating)),
];
