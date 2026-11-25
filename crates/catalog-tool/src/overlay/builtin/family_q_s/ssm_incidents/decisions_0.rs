use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchGetIncidentFindings",
        Decision::Deferred(
            "Required selectors incidentRecordArn, findingIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findings, errors. Pinned purpose: Retrieves details about all specified findings for an incident, including descriptive details about each finding. A finding represents a recent application environment change made by an CodeDeploy deployment or an CloudFormation stack creation or upd",
        ),
    ),
    (
        "CreateReplicationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateResponsePlan", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTimelineEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteIncidentRecord",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteReplicationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteResponsePlan", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTimelineEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetIncidentRecord",
        Decision::Deferred(
            "Required selectors arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members incidentRecord. Pinned purpose: Returns the details for the specified incident record.",
        ),
    ),
    (
        "GetReplicationSet",
        Decision::Deferred(
            "Required selectors arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members replicationSet. Pinned purpose: Retrieve your Incident Manager replication set.",
        ),
    ),
    (
        "GetResourcePolicies",
        Decision::Deferred(
            "Required selectors resourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourcePolicies, nextToken. Pinned purpose: Retrieves the resource policies attached to the specified response plan.",
        ),
    ),
    (
        "GetResponsePlan",
        Decision::Deferred(
            "Required selectors arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members arn, name, displayName, incidentTemplate, chatChannel, engagements, actions, integrations. Pinned purpose: Retrieves the details of the specified response plan.",
        ),
    ),
    (
        "GetTimelineEvent",
        Decision::Deferred(
            "Required selectors incidentRecordArn, eventId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members event. Pinned purpose: Retrieves a timeline event based on its ID and incident record.",
        ),
    ),
    (
        "ListIncidentFindings",
        Decision::Deferred(
            "Required selectors incidentRecordArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findings, nextToken. Pinned purpose: Retrieves a list of the IDs of findings, plus their last modified times, that have been identified for a specified incident. A finding represents a recent application environment change made by an CloudFormation stack creation or update or an CodeDep",
        ),
    ),
    (
        "ListRelatedItems",
        Decision::Deferred(
            "Required selectors incidentRecordArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members relatedItems, nextToken. Pinned purpose: List all related items for an incident record.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags that are attached to the specified response plan or incident.",
        ),
    ),
    (
        "ListTimelineEvents",
        Decision::Deferred(
            "Required selectors incidentRecordArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members eventSummaries, nextToken. Pinned purpose: Lists timeline events for the specified incident record.",
        ),
    ),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    ("StartIncident", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDeletionProtection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateIncidentRecord",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateRelatedItems", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateReplicationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateResponsePlan", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateTimelineEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
];
