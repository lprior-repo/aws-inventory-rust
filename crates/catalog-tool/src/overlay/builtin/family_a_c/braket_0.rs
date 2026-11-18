// Pinned source: .local/aws-models/models/braket/service/2019-09-01/braket-2019-09-01.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelJob",
            deferred(
                "Model requires jobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels an Amazon Braket hybrid job.",
            ),
        ),
        (
            "CancelQuantumTask",
            deferred(
                "Model requires quantumTaskArn, clientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels the specified task.",
            ),
        ),
        (
            "CreateJob",
            deferred(
                "Model requires clientToken, algorithmSpecification, outputDataConfig, jobName, roleArn, instanceConfig, deviceConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Braket hybrid job.",
            ),
        ),
        (
            "CreateQuantumTask",
            deferred(
                "Model requires clientToken, deviceArn, shots, outputS3Bucket, outputS3KeyPrefix, action. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a quantum task.",
            ),
        ),
        (
            "CreateSpendingLimit",
            deferred(
                "Model requires clientToken, deviceArn, spendingLimit. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a spending limit for a specified quantum device. Spending limits help you control costs by setting maximum amounts that can be spent on quantum computing tasks within a specified time period. Simulators do not support…",
            ),
        ),
        (
            "DeleteSpendingLimit",
            deferred(
                "Model requires spendingLimitArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing spending limit. This operation permanently removes the spending limit and cannot be undone. After deletion, the associated device becomes unrestricted for spending.",
            ),
        ),
        (
            "GetDevice",
            deferred(
                "Model requires deviceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the devices available in Amazon Braket. For backwards compatibility with older versions of BraketSchemas, OpenQASM information is omitted from GetDevice API calls. To get this information the user-agent needs to present…",
            ),
        ),
        (
            "GetJob",
            deferred(
                "Model requires jobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the specified Amazon Braket hybrid job.",
            ),
        ),
        (
            "GetQuantumTask",
            deferred(
                "Model requires quantumTaskArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the specified quantum task.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Shows the tags associated with this resource.",
            ),
        ),
        (
            "SearchDevices",
            deferred(
                "Requires filters, a list of SearchDevicesFilter structures. An unfiltered search needs an empty structure-list; FixedValue currently supports only scalar values and string lists, not a typed empty structure-list.",
            ),
        ),
        (
            "SearchJobs",
            deferred(
                "Model requires filters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for Amazon Braket hybrid jobs that match the specified filter values.",
            ),
        ),
        (
            "SearchQuantumTasks",
            deferred(
                "Model requires filters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for tasks that match the specified filter values.",
            ),
        ),
        (
            "SearchSpendingLimits",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is spendingLimits; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Searches and lists spending limits based on specified filters. This operation supports pagination and allows filtering by various criteria to find specific spending limits. We recommend using pagination to ensure that the…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add a tag to the specified resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove tags from a resource.",
            ),
        ),
        (
            "UpdateSpendingLimit",
            deferred(
                "Model requires spendingLimitArn, clientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing spending limit. You can modify the spending amount or time period. Changes take effect immediately.",
            ),
        ),
    ])
}
