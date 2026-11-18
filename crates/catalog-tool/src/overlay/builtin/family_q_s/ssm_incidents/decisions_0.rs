// Pinned operation purposes: .local/aws-models/models/ssm-incidents/service/2018-05-10/ssm-incidents-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchGetIncidentFindings",
        Decision::Deferred(
            "Required selectors incidentRecordArn, findingIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findings, errors. Pinned purpose: Retrieves details about all specified findings for an incident, including descriptive details about each finding. A finding represents a recent application environment change made by an CodeDeploy deployment or an CloudFormation stack creation or upd",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: A replication set replicates and encrypts your data to the provided Regions with the provided KMS key.
    (
        "CreateReplicationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a response plan that automates the initial response to incidents. A response plan engages contacts, starts chat channel collaboration, and initiates runbooks at the beginning of an incident.
    ("CreateResponsePlan", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a custom timeline event on the incident details page of an incident record. Incident Manager automatically creates timeline events that mark key moments during an incident. You can create custom timeline events to mark important events that Incident Manager can detect automatically.
    (
        "CreateTimelineEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Delete an incident record from Incident Manager.
    (
        "DeleteIncidentRecord",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes all Regions in your replication set. Deleting the replication set deletes all Incident Manager data.
    (
        "DeleteReplicationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the resource policy that Resource Access Manager uses to share your Incident Manager resource.
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified response plan. Deleting a response plan stops all linked CloudWatch alarms and EventBridge events from creating an incident with this response plan.
    ("DeleteResponsePlan", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a timeline event from an incident.
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds a resource policy to the specified response plan. The resource policy is used to share the response plan using Resource Access Manager (RAM). For more information about cross-account sharing, see Cross-Region and cross-account incident management .
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Used to start an incident from CloudWatch alarms, EventBridge events, or manually.
    ("StartIncident", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a tag to a response plan.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Update deletion protection to either allow or deny deletion of the final Region in a replication set.
    (
        "UpdateDeletionProtection",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update the details of an incident record. You can use this operation to update an incident record from the defined chat channel. For more information about using actions in chat channels, see Interacting through chat .
    (
        "UpdateIncidentRecord",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Add or remove related items from the related items tab of an incident record.
    ("UpdateRelatedItems", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Add or delete Regions from your replication set.
    (
        "UpdateReplicationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified response plan.
    ("UpdateResponsePlan", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a timeline event. You can update events of type Custom Event .
    (
        "UpdateTimelineEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
];
