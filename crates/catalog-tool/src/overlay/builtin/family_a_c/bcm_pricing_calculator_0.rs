use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchCreateBillScenarioCommitmentModification",
            deferred(
                "Model requires billScenarioId, commitmentModifications. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create Compute Savings Plans, EC2 Instance Savings Plans, or EC2 Reserved Instances commitments that you want to model in a Bill Scenario. The BatchCreateBillScenarioCommitmentModification operation doesn't have its own IAM…",
            ),
        ),
        (
            "BatchCreateBillScenarioUsageModification",
            deferred(
                "Model requires billScenarioId, usageModifications. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create Amazon Web Services service usage that you want to model in a Bill Scenario. The BatchCreateBillScenarioUsageModification operation doesn't have its own IAM permission. To authorize this operation for Amazon Web Services…",
            ),
        ),
        (
            "BatchCreateWorkloadEstimateUsage",
            deferred(
                "Model requires workloadEstimateId, usage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create Amazon Web Services service usage that you want to model in a Workload Estimate. The BatchCreateWorkloadEstimateUsage operation doesn't have its own IAM permission. To authorize this operation for Amazon Web Services…",
            ),
        ),
        (
            "BatchDeleteBillScenarioCommitmentModification",
            deferred(
                "Model requires billScenarioId, ids. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete commitment that you have created in a Bill Scenario. You can only delete a commitment that you had added and cannot model deletion (or removal) of a existing commitment. If you want model deletion of an existing…",
            ),
        ),
        (
            "BatchDeleteBillScenarioUsageModification",
            deferred(
                "Model requires billScenarioId, ids. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete usage that you have created in a Bill Scenario. You can only delete usage that you had added and cannot model deletion (or removal) of a existing usage. If you want model removal of an existing usage, see…",
            ),
        ),
        (
            "BatchDeleteWorkloadEstimateUsage",
            deferred(
                "Model requires workloadEstimateId, ids. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete usage that you have created in a Workload estimate. You can only delete usage that you had added and cannot model deletion (or removal) of a existing usage. If you want model removal of an existing usage, see…",
            ),
        ),
        (
            "BatchUpdateBillScenarioCommitmentModification",
            deferred(
                "Model requires billScenarioId, commitmentModifications. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update a newly added or existing commitment. You can update the commitment group based on a commitment ID and a Bill scenario ID. The BatchUpdateBillScenarioCommitmentModification operation doesn't have its own IAM permission. To…",
            ),
        ),
        (
            "BatchUpdateBillScenarioUsageModification",
            deferred(
                "Model requires billScenarioId, usageModifications. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update a newly added or existing usage lines. You can update the usage amounts, usage hour, and usage group based on a usage ID and a Bill scenario ID. The BatchUpdateBillScenarioUsageModification operation doesn't have its own…",
            ),
        ),
        (
            "BatchUpdateWorkloadEstimateUsage",
            deferred(
                "Model requires workloadEstimateId, usage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update a newly added or existing usage lines. You can update the usage amounts and usage group based on a usage ID and a Workload estimate ID. The BatchUpdateWorkloadEstimateUsage operation doesn't have its own IAM permission. To…",
            ),
        ),
        (
            "CreateBillEstimate",
            deferred(
                "Model requires billScenarioId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create a Bill estimate from a Bill scenario. In the Bill scenario you can model usage addition, usage changes, and usage removal. You can also model commitment addition and commitment removal. After all changes in a Bill scenario…",
            ),
        ),
        (
            "CreateBillScenario",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new bill scenario to model potential changes to Amazon Web Services usage and costs.",
            ),
        ),
        (
            "CreateWorkloadEstimate",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new workload estimate to model costs for a specific workload.",
            ),
        ),
        (
            "DeleteBillEstimate",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing bill estimate.",
            ),
        ),
        (
            "DeleteBillScenario",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing bill scenario.",
            ),
        ),
        (
            "DeleteWorkloadEstimate",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing workload estimate.",
            ),
        ),
        (
            "GetBillEstimate",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details of a specific bill estimate.",
            ),
        ),
        (
            "GetBillScenario",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details of a specific bill scenario.",
            ),
        ),
        (
            "GetPreferences",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is managementAccountRateTypeSelections, memberAccountRateTypeSelections, standaloneAccountRateTypeSelections; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves the current preferences for Pricing Calculator.",
            ),
        ),
        (
            "GetWorkloadEstimate",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details of a specific workload estimate.",
            ),
        ),
        (
            "ListBillEstimateCommitments",
            deferred(
                "Model requires billEstimateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the commitments associated with a bill estimate.",
            ),
        ),
        (
            "ListBillEstimateInputCommitmentModifications",
            deferred(
                "Model requires billEstimateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the input commitment modifications associated with a bill estimate.",
            ),
        ),
        (
            "ListBillEstimateInputUsageModifications",
            deferred(
                "Model requires billEstimateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the input usage modifications associated with a bill estimate.",
            ),
        ),
        (
            "ListBillEstimateLineItems",
            deferred(
                "Model requires billEstimateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the line items associated with a bill estimate.",
            ),
        ),
        (
            "ListBillEstimates",
            OperationOverlay {
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Account,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListBillScenarioCommitmentModifications",
            deferred(
                "Model requires billScenarioId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the commitment modifications associated with a bill scenario.",
            ),
        ),
        (
            "ListBillScenarioUsageModifications",
            deferred(
                "Model requires billScenarioId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the usage modifications associated with a bill scenario.",
            ),
        ),
        (
            "ListBillScenarios",
            OperationOverlay {
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Account,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all tags associated with a specified resource.",
            ),
        ),
        (
            "ListWorkloadEstimateUsage",
            deferred(
                "Model requires workloadEstimateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the usage associated with a workload estimate.",
            ),
        ),
        (
            "ListWorkloadEstimates",
            OperationOverlay {
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Account,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "TagResource",
            deferred(
                "Model requires arn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds one or more tags to a specified resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires arn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from a specified resource.",
            ),
        ),
        (
            "UpdateBillEstimate",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing bill estimate.",
            ),
        ),
        (
            "UpdateBillScenario",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing bill scenario.",
            ),
        ),
        (
            "UpdatePreferences",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is managementAccountRateTypeSelections, memberAccountRateTypeSelections, standaloneAccountRateTypeSelections; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Updates the preferences for Pricing Calculator.",
            ),
        ),
        (
            "UpdateWorkloadEstimate",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing workload estimate.",
            ),
        ),
    ])
}
