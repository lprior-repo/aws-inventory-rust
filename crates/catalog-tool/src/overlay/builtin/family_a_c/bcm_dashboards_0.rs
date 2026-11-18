// Pinned source: .local/aws-models/models/bcm-dashboards/service/2025-08-18/bcm-dashboards-2025-08-18.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateDashboard",
            deferred(
                "Model requires name, widgets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new dashboard that can contain multiple widgets displaying cost and usage data. You can add custom widgets or use predefined widgets, arranging them in your preferred layout.",
            ),
        ),
        (
            "CreateScheduledReport",
            deferred(
                "Model requires scheduledReport. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new scheduled report for a dashboard. A scheduled report automatically generates and delivers dashboard snapshots on a recurring schedule. Reports are delivered within 15 minutes of the scheduled delivery time.",
            ),
        ),
        (
            "DeleteDashboard",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specified dashboard. This action cannot be undone.",
            ),
        ),
        (
            "DeleteScheduledReport",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specified scheduled report. This is an irreversible operation.",
            ),
        ),
        (
            "ExecuteScheduledReport",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Triggers an immediate execution of a scheduled report, outside of its regular schedule. The scheduled report must be in ENABLED state. Calling this operation on a DISABLED scheduled report returns a ValidationException . If a…",
            ),
        ),
        (
            "GetDashboard",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the configuration and metadata of a specified dashboard, including its widgets and layout settings.",
            ),
        ),
        (
            "GetResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the resource-based policy attached to a dashboard, showing sharing configurations and permissions.",
            ),
        ),
        (
            "GetScheduledReport",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the configuration and metadata of a specified scheduled report.",
            ),
        ),
        // Model: Returns a list of all dashboards in your account.
        (
            "ListDashboards",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "dashboards",
                    Identity::Arn(path("arn")?),
                    Scope::Account,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of scheduled reports in your account.
        (
            "ListScheduledReports",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "scheduledReports",
                    Identity::Arn(path("arn")?),
                    Scope::Account,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of all tags associated with a specified dashboard resource.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, resourceTags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates tags for a specified dashboard resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, resourceTagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes specified tags from a dashboard resource.",
            ),
        ),
        (
            "UpdateDashboard",
            deferred(
                "Model requires arn, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing dashboard's properties, including its name, description, and widget configurations.",
            ),
        ),
        (
            "UpdateScheduledReport",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing scheduled report's properties, including its name, description, schedule configuration, and widget settings. Only the parameters included in the request are updated; all other properties remain unchanged.",
            ),
        ),
    ])
}
