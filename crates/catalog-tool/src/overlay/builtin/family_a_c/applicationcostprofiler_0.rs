// Pinned source: .local/aws-models/models/applicationcostprofiler/service/2020-09-10/applicationcostprofiler-2020-09-10.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteReportDefinition",
            deferred(
                "Model requires reportId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified report definition in AWS Application Cost Profiler. This stops the report from being generated.",
            ),
        ),
        // Model: Retrieves the definition of a report already configured in AWS Application Cost Profiler.
        (
            "GetReportDefinition",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("reportId")?,
                    Binding::From {
                        operation: name("ListReportDefinitions")?,
                        path: path("reportId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("reportId")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("reportId")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "ImportApplicationUsage",
            deferred(
                "Model requires sourceS3Location. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ingests application usage data from Amazon Simple Storage Service (Amazon S3). The data must already exist in the S3 location. As part of the action, AWS Application Cost Profiler copies the object from your S3 bucket to an S3…",
            ),
        ),
        // Model: Retrieves a list of all reports and their configurations for your AWS account. The maximum number of reports is one.
        (
            "ListReportDefinitions",
            OperationOverlay {
                queryable: vec![path("reportId")?],
                ..read(
                    "reportDefinitions",
                    Identity::Key(vec![path("reportId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "PutReportDefinition",
            deferred(
                "Model requires reportId, reportDescription, reportFrequency, format, destinationS3Location. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates the report definition for a report in Application Cost Profiler.",
            ),
        ),
        (
            "UpdateReportDefinition",
            deferred(
                "Model requires reportId, reportDescription, reportFrequency, format, destinationS3Location. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates existing report in AWS Application Cost Profiler.",
            ),
        ),
    ])
}
