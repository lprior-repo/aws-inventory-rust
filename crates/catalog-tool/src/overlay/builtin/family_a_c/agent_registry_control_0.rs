use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateRegistry",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new registry, a catalog that organizes registry records and defines their discovery authorization and record approval behavior. Creation is asynchronous: the registry begins in the CREATING status and becomes usable…",
            ),
        ),
        (
            "CreateRegistryRecord",
            deferred(
                "Model requires registryId, name, recordType, descriptors. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a registry record within a registry. A registry record describes a discoverable resource, such as an MCP server, an agent, an agent skill, or a custom resource. Creation is asynchronous: the record is returned with the…",
            ),
        ),
        (
            "DeleteRegistry",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a registry. Deletion is asynchronous: the registry transitions to the DELETING status and is removed along with its registry records.",
            ),
        ),
        (
            "DeleteRegistryRecord",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a registry record",
            ),
        ),
        ("GetRegistry", denied(DenyReason::Content)),
        (
            "GetRegistryRecord",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of a registry record",
            ),
        ),
        (
            "ListRegistries",
            OperationOverlay {
                queryable: vec![path("registryArn")?],
                ..read(
                    "registries",
                    Identity::Arn(path("registryArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListRegistryRecords",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the registry records within a registry, with optional filtering by name, status, and record type",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags associated with the specified Amazon Web Services Agent Registry resource. Returns the current tag key-value pairs on the resource.",
            ),
        ),
        (
            "SubmitRegistryRecordForApproval",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits a DRAFT registry record for approval, moving it into the registry's approval workflow. Depending on the registry's approval configuration, the record is either auto-approved or set to PENDING_APPROVAL for a curator to…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or overwrites one or more tags for the specified Amazon Web Services Agent Registry resource. Tags are key-value pairs that you can use to categorize and manage Amazon Web Services resources. If a tag with the same key…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from the specified Amazon Web Services Agent Registry resource. The operation removes only the tags whose keys you supply; other tags on the resource remain unchanged.",
            ),
        ),
        (
            "UpdateRegistry",
            deferred(
                "Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing registry. This operation uses PATCH semantics: specify only the fields you want to change, and omit the rest to leave them unchanged. Updates are applied asynchronously and the registry transitions to the…",
            ),
        ),
        (
            "UpdateRegistryRecord",
            deferred(
                "Model requires registryId, recordId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a registry record. The update is asynchronous: the record is returned with the UPDATING status while it is processed. Fields that use update wrappers follow PATCH semantics: omit the field to leave it unchanged.",
            ),
        ),
        (
            "UpdateRegistryRecordStatus",
            deferred(
                "Model requires registryId, recordId, status, statusReason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of a registry record as part of the registry's curation workflow, for example to approve or reject a record that is pending approval, or to deprecate an approved record so that it is no longer discoverable",
            ),
        ),
    ])
}
