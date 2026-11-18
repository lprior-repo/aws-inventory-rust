// Pinned source: .local/aws-models/models/bcm-data-exports/service/2023-11-26/bcm-data-exports-2023-11-26.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateExport",
            deferred(
                "Model requires Export. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a data export and specifies the data query, the delivery preference, and any optional resource tags. A DataQuery consists of both a QueryStatement and TableConfigurations . The QueryStatement is an SQL statement. Data…",
            ),
        ),
        (
            "DeleteExport",
            deferred(
                "Model requires ExportArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing data export.",
            ),
        ),
        (
            "GetExecution",
            deferred(
                "Model requires ExportArn, ExecutionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports data based on the source data update.",
            ),
        ),
        (
            "GetExport",
            deferred(
                "Model requires ExportArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Views the definition of an existing data export.",
            ),
        ),
        (
            "GetTable",
            deferred(
                "Model requires TableName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the metadata for the specified table and table properties. This includes the list of columns in the table schema, their data types, and column descriptions.",
            ),
        ),
        (
            "ListExecutions",
            deferred(
                "Model requires ExportArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the historical executions for the export.",
            ),
        ),
        // Model: Lists all data export definitions.
        (
            "ListExports",
            OperationOverlay {
                queryable: vec![path("ExportArn")?],
                ..read(
                    "Exports",
                    Identity::Arn(path("ExportArn")?),
                    Scope::Account,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTables",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Tables; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all available tables in data exports.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List tags associated with an existing data export.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, ResourceTags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds tags for an existing data export definition.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, ResourceTagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes tags associated with an existing data export definition.",
            ),
        ),
        (
            "UpdateExport",
            deferred(
                "Model requires ExportArn, Export. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing data export by overwriting all export parameters. All export parameters must be provided in the UpdateExport request.",
            ),
        ),
    ])
}
