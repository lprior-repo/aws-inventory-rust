use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteReportDefinition",
            deferred(
                "Model requires reportId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified report definition in AWS Application Cost Profiler. This stops the report from being generated.",
            ),
        ),
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
