// Pinned source: .local/aws-models/models/amplify/service/2017-07-25/amplify-2017-07-25.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateApp",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Amplify app.",
            ),
        ),
        (
            "CreateBackendEnvironment",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new backend environment for an Amplify app. This API is available only to Amplify Gen 1 applications where the backend is created using Amplify Studio or the Amplify command line interface (CLI). This API isn’t…",
            ),
        ),
        (
            "CreateBranch",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new branch for an Amplify app.",
            ),
        ),
        (
            "CreateDeployment",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a deployment for a manually deployed Amplify app. Manually deployed apps are not connected to a Git repository. The maximum duration between the CreateDeployment call and the StartDeployment call cannot exceed 8 hours. If…",
            ),
        ),
        (
            "CreateDomainAssociation",
            deferred(
                "Model requires appId, domainName, subDomainSettings. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new domain association for an Amplify app. This action associates a custom domain with the Amplify app",
            ),
        ),
        (
            "CreateWebhook",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new webhook on an Amplify app.",
            ),
        ),
        (
            "DeleteApp",
            deferred(
                "Model requires appId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing Amplify app specified by an app ID.",
            ),
        ),
        (
            "DeleteBackendEnvironment",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a backend environment for an Amplify app. This API is available only to Amplify Gen 1 applications where the backend is created using Amplify Studio or the Amplify command line interface (CLI). This API isn’t available to…",
            ),
        ),
        (
            "DeleteBranch",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a branch for an Amplify app.",
            ),
        ),
        (
            "DeleteDomainAssociation",
            deferred(
                "Model requires appId, domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a domain association for an Amplify app.",
            ),
        ),
        (
            "DeleteJob",
            deferred(
                "Model requires appId, branchName, jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a job for a branch of an Amplify app.",
            ),
        ),
        (
            "DeleteWebhook",
            deferred(
                "Model requires webhookId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a webhook.",
            ),
        ),
        (
            "GenerateAccessLogs",
            deferred(
                "Model requires domainName, appId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the website access logs for a specific time range using a presigned URL.",
            ),
        ),
        // Model: Returns an existing Amplify app specified by an app ID.
        ("GetApp", denied(DenyReason::Secret)),
        (
            "GetArtifactUrl",
            deferred(
                "Model requires artifactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the artifact info that corresponds to an artifact id.",
            ),
        ),
        (
            "GetBackendEnvironment",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a backend environment for an Amplify app. This API is available only to Amplify Gen 1 applications where the backend is created using Amplify Studio or the Amplify command line interface (CLI). This API isn’t available to…",
            ),
        ),
        // Model: Returns a branch for an Amplify app.
        ("GetBranch", denied(DenyReason::Secret)),
        (
            "GetDomainAssociation",
            deferred(
                "Model requires appId, domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the domain information for an Amplify app.",
            ),
        ),
        (
            "GetJob",
            deferred(
                "Model requires appId, branchName, jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a job for a branch of an Amplify app.",
            ),
        ),
        (
            "GetWebhook",
            deferred(
                "Model requires webhookId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the webhook information that corresponds to a specified webhook ID.",
            ),
        ),
        // Model: Returns a list of the existing Amplify apps.
        ("ListApps", denied(DenyReason::Secret)),
        (
            "ListArtifacts",
            deferred(
                "Model requires appId, branchName, jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of end-to-end testing artifacts for a specified app, branch, and job. To return the build artifacts, use the GetJob API. For more information about Amplify testing support, see Setting up end-to-end Cypress tests…",
            ),
        ),
        (
            "ListBackendEnvironments",
            deferred(
                "Model requires appId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the backend environments for an Amplify app. This API is available only to Amplify Gen 1 applications where the backend is created using Amplify Studio or the Amplify command line interface (CLI). This API isn’t available…",
            ),
        ),
        // Model: Lists the branches of an Amplify app.
        ("ListBranches", denied(DenyReason::Secret)),
        (
            "ListDomainAssociations",
            deferred(
                "Model requires appId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the domain associations for an Amplify app.",
            ),
        ),
        (
            "ListJobs",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the jobs for a branch of an Amplify app.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags for a specified Amazon Resource Name (ARN).",
            ),
        ),
        (
            "ListWebhooks",
            deferred(
                "Model requires appId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of webhooks for an Amplify app.",
            ),
        ),
        (
            "StartDeployment",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a deployment for a manually deployed app. Manually deployed apps are not connected to a Git repository. The maximum duration between the CreateDeployment call and the StartDeployment call cannot exceed 8 hours. If the…",
            ),
        ),
        (
            "StartJob",
            deferred(
                "Model requires appId, branchName, jobType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a new job for a branch of an Amplify app.",
            ),
        ),
        (
            "StopJob",
            deferred(
                "Model requires appId, branchName, jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a job that is in progress for a branch of an Amplify app.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tags the resource with a tag key and value.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Untags a resource with a specified Amazon Resource Name (ARN).",
            ),
        ),
        (
            "UpdateApp",
            deferred(
                "Model requires appId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Amplify app.",
            ),
        ),
        (
            "UpdateBranch",
            deferred(
                "Model requires appId, branchName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a branch for an Amplify app.",
            ),
        ),
        (
            "UpdateDomainAssociation",
            deferred(
                "Model requires appId, domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new domain association for an Amplify app.",
            ),
        ),
        (
            "UpdateWebhook",
            deferred(
                "Model requires webhookId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a webhook.",
            ),
        ),
    ])
}
