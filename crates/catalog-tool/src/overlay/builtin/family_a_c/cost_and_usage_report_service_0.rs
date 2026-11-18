// Pinned source: .local/aws-models/models/cost-and-usage-report-service/service/2017-01-06/cost-and-usage-report-service-2017-01-06.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteReportDefinition",
            deferred(
                "Model requires ReportName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified report. Any tags associated with the report are also deleted.",
            ),
        ),
        // Model: Lists the Amazon Web Services Cost and Usage Report available to this account.
        (
            "DescribeReportDefinitions",
            OperationOverlay {
                queryable: vec![path("ReportName")?],
                ..read(
                    "ReportDefinitions",
                    Identity::Key(vec![path("ReportName")?]),
                    Scope::Account,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ReportName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags associated with the specified report definition.",
            ),
        ),
        (
            "ModifyReportDefinition",
            deferred(
                "Model requires ReportName, ReportDefinition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows you to programmatically update your report preferences.",
            ),
        ),
        (
            "PutReportDefinition",
            deferred(
                "Model requires ReportDefinition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new report using the description that you provide.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ReportName, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of tags with a report definition.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ReportName, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a set of tags from a report definition.",
            ),
        ),
    ])
}
