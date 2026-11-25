// Pinned source: .local/aws-models/models/codebuild/service/2016-10-06/codebuild-2016-10-06.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchDeleteBuilds",
            deferred(
                "Model requires ids. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes one or more builds.",
            ),
        ),
        // Model: Retrieves information about one or more batch builds.
        ("BatchGetBuildBatches", denied(DenyReason::Secret)),
        // Model: Gets information about one or more builds.
        ("BatchGetBuilds", denied(DenyReason::Secret)),
        (
            "BatchGetCommandExecutions",
            deferred(
                "Model requires sandboxId, commandExecutionIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about the command executions.",
            ),
        ),
        (
            "BatchGetFleets",
            deferred(
                "Model requires names. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more compute fleets.",
            ),
        ),
        // Model: Gets information about one or more build projects.
        ("BatchGetProjects", denied(DenyReason::Secret)),
        (
            "BatchGetReportGroups",
            deferred(
                "Model requires reportGroupArns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an array of report groups.",
            ),
        ),
        (
            "BatchGetReports",
            deferred(
                "Model requires reportArns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an array of reports.",
            ),
        ),
        (
            "BatchGetSandboxes",
            deferred(
                "Model requires ids. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about the sandbox status.",
            ),
        ),
        (
            "CreateFleet",
            deferred(
                "Model requires name, baseCapacity, environmentType, computeType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a compute fleet.",
            ),
        ),
        (
            "CreateProject",
            deferred(
                "Model requires name, source, artifacts, environment, serviceRole. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a build project.",
            ),
        ),
        (
            "CreateReportGroup",
            deferred(
                "Model requires name, type, exportConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a report group. A report group contains a collection of reports.",
            ),
        ),
        (
            "CreateWebhook",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: For an existing CodeBuild build project that has its source code stored in a GitHub or Bitbucket repository, enables CodeBuild to start rebuilding the source code every time a code change is pushed to the repository. If you…",
            ),
        ),
        (
            "DeleteBuildBatch",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a batch build.",
            ),
        ),
        (
            "DeleteFleet",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a compute fleet. When you delete a compute fleet, its builds are not deleted.",
            ),
        ),
        (
            "DeleteProject",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a build project. When you delete a project, its builds are not deleted.",
            ),
        ),
        (
            "DeleteReport",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a report.",
            ),
        ),
        (
            "DeleteReportGroup",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a report group. Before you delete a report group, you must delete its reports.",
            ),
        ),
        (
            "DeleteResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a resource policy that is identified by its resource ARN.",
            ),
        ),
        (
            "DeleteSourceCredentials",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a set of GitHub, GitHub Enterprise, or Bitbucket source credentials.",
            ),
        ),
        (
            "DeleteWebhook",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: For an existing CodeBuild build project that has its source code stored in a GitHub or Bitbucket repository, stops CodeBuild from rebuilding the source code every time a code change is pushed to the repository.",
            ),
        ),
        (
            "DescribeCodeCoverages",
            deferred(
                "Model requires reportArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves one or more code coverage reports.",
            ),
        ),
        (
            "DescribeTestCases",
            deferred(
                "Model requires reportArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of details about test cases for a report.",
            ),
        ),
        (
            "GetReportGroupTrend",
            deferred(
                "Model requires reportGroupArn, trendField. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Analyzes and accumulates test report values for the specified test reports.",
            ),
        ),
        (
            "GetResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a resource policy that is identified by its resource ARN.",
            ),
        ),
        (
            "ImportSourceCredentials",
            deferred(
                "Model requires token, serverType, authType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports the source repository credentials for an CodeBuild project that has its source code stored in a GitHub, GitHub Enterprise, GitLab, GitLab Self Managed, or Bitbucket repository.",
            ),
        ),
        (
            "InvalidateProjectCache",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resets the cache for a project.",
            ),
        ),
        // Model: Retrieves the identifiers of your build batches in the current region.
        (
            "ListBuildBatches",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "ids",
                    Identity::Key(vec![path("$")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListBuildBatchesForProject",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ids; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves the identifiers of the build batches for a specific project.",
            ),
        ),
        // Model: Gets a list of build IDs, with each build ID representing a single build.
        (
            "ListBuilds",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "ids",
                    Identity::Key(vec![path("$")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListBuildsForProject",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a list of build identifiers for the specified build project, with each build identifier representing a single build.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Gets a list of command executions for a sandbox.
        (
            "ListCommandExecutionsForSandbox",
            denied(DenyReason::Content),
        ),
        (
            "ListCuratedEnvironmentImages",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is platforms; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets information about Docker images that are managed by CodeBuild.",
            ),
        ),
        // Model: Gets a list of compute fleet names with each compute fleet name representing a single compute fleet.
        (
            "ListFleets",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "fleets",
                    Identity::Key(vec![path("$")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Gets a list of build project names, with each build project name representing a single build project.
        (
            "ListProjects",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "projects",
                    Identity::Template(
                        "arn:{partition}:codebuild:{region}:{account}:project/{$}".to_owned(),
                    ),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Gets a list ARNs for the report groups in the current Amazon Web Services account.
        (
            "ListReportGroups",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "reportGroups",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of ARNs for the reports in the current Amazon Web Services account.
        (
            "ListReports",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "reports",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListReportsForReportGroup",
            deferred(
                "Model requires reportGroupArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of ARNs for the reports that belong to a ReportGroup .",
            ),
        ),
        // Model: Gets a list of sandboxes.
        (
            "ListSandboxes",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "ids",
                    Identity::Key(vec![path("$")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListSandboxesForProject",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a list of sandboxes for a given project.",
            ),
        ),
        (
            "ListSharedProjects",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is projects; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets a list of projects that are shared with other Amazon Web Services accounts or users.",
            ),
        ),
        (
            "ListSharedReportGroups",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is reportGroups; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets a list of report groups that are shared with other Amazon Web Services accounts or users.",
            ),
        ),
        (
            "ListSourceCredentials",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "sourceCredentialsInfos",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "PutResourcePolicy",
            deferred(
                "Model requires policy, resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stores a resource policy for the ARN of a Project or ReportGroup object.",
            ),
        ),
        (
            "RetryBuild",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is build; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Restarts a build.",
            ),
        ),
        (
            "RetryBuildBatch",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is buildBatch; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Restarts a failed batch build. Only batch builds that have failed can be retried.",
            ),
        ),
        (
            "StartBuild",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts running a build with the settings defined in the project. These setting include: how to run a build, where to get the source code, which build environment to use, which build commands to run, and where to store the build…",
            ),
        ),
        (
            "StartBuildBatch",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a batch build for a project.",
            ),
        ),
        (
            "StartCommandExecution",
            deferred(
                "Model requires sandboxId, command. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a command execution.",
            ),
        ),
        (
            "StartSandbox",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is sandbox; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Starts a sandbox.",
            ),
        ),
        (
            "StartSandboxConnection",
            deferred(
                "Model requires sandboxId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a sandbox connection.",
            ),
        ),
        (
            "StopBuild",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attempts to stop running a build.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "StopBuildBatch",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a running batch build.",
            ),
        ),
        (
            "StopSandbox",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a sandbox.",
            ),
        ),
        (
            "UpdateFleet",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a compute fleet.",
            ),
        ),
        (
            "UpdateProject",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes the settings of a build project.",
            ),
        ),
        (
            "UpdateProjectVisibility",
            deferred(
                "Model requires projectArn, projectVisibility. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes the public visibility for a project. The project's build results, logs, and artifacts are available to the general public. For more information, see Public build projects in the CodeBuild User Guide . The following should…",
            ),
        ),
        (
            "UpdateReportGroup",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a report group.",
            ),
        ),
        (
            "UpdateWebhook",
            deferred(
                "Model requires projectName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the webhook associated with an CodeBuild build project. If you use Bitbucket for your repository, rotateSecret is ignored.",
            ),
        ),
    ])
}
