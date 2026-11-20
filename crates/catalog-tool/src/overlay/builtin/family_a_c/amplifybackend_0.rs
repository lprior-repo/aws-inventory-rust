// Pinned source: .local/aws-models/models/amplifybackend/service/2020-08-11/amplifybackend-2020-08-11.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CloneBackend",
            deferred(
                "Model requires AppId, BackendEnvironmentName, TargetEnvironmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation clones an existing backend.",
            ),
        ),
        (
            "CreateBackend",
            deferred(
                "Model requires AppId, AppName, BackendEnvironmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation creates a backend for an Amplify app. Backends are automatically created at the time of app creation.",
            ),
        ),
        (
            "CreateBackendAPI",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceConfig, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new backend API resource.",
            ),
        ),
        (
            "CreateBackendAuth",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceConfig, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new backend authentication resource.",
            ),
        ),
        (
            "CreateBackendConfig",
            deferred(
                "Model requires AppId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a config object for a backend.",
            ),
        ),
        (
            "CreateBackendStorage",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceConfig, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a backend storage resource.",
            ),
        ),
        (
            "CreateToken",
            deferred(
                "Model requires AppId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Generates a one-time challenge code to authenticate a user into your Amplify Admin UI.",
            ),
        ),
        (
            "DeleteBackend",
            deferred(
                "Model requires AppId, BackendEnvironmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an existing environment from your Amplify project.",
            ),
        ),
        (
            "DeleteBackendAPI",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing backend API resource.",
            ),
        ),
        (
            "DeleteBackendAuth",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing backend authentication resource.",
            ),
        ),
        (
            "DeleteBackendStorage",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName, ServiceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified backend storage resource.",
            ),
        ),
        (
            "DeleteToken",
            deferred(
                "Model requires AppId, SessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the challenge token based on the given appId and sessionId.",
            ),
        ),
        (
            "GenerateBackendAPIModels",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Generates a model schema for an existing backend API resource.",
            ),
        ),
        (
            "GetBackend",
            deferred(
                "Model requires AppId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides project-level details for your Amplify UI project.",
            ),
        ),
        (
            "GetBackendAPI",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the details for a backend API.",
            ),
        ),
        (
            "GetBackendAPIModels",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a model introspection schema for an existing backend API resource.",
            ),
        ),
        (
            "GetBackendAuth",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a backend auth details.",
            ),
        ),
        (
            "GetBackendJob",
            deferred(
                "Model requires AppId, BackendEnvironmentName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specific job.",
            ),
        ),
        (
            "GetBackendStorage",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details for a backend storage resource.",
            ),
        ),
        (
            "GetToken",
            deferred(
                "Model requires AppId, SessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the challenge token based on the given appId and sessionId.",
            ),
        ),
        (
            "ImportBackendAuth",
            deferred(
                "Model requires AppId, BackendEnvironmentName, NativeClientId, UserPoolId, WebClientId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports an existing backend authentication resource.",
            ),
        ),
        (
            "ImportBackendStorage",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ServiceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports an existing backend storage resource.",
            ),
        ),
        (
            "ListBackendJobs",
            deferred(
                "Model requires AppId, BackendEnvironmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the jobs for the backend of an Amplify app.",
            ),
        ),
        // Model: The list of S3 buckets in your account. Bucket names are partition-unique, so the
        // row shares s3.ListBuckets' ARN identity domain for AWS::S3::Bucket.
        (
            "ListS3Buckets",
            OperationOverlay {
                queryable: vec![path("Name")?],
                ..read(
                    "Buckets",
                    Identity::Template("arn:{partition}:s3:::{Name}".to_owned()),
                    Scope::Account,
                    Paginator::Token(crate::overlay::TokenPair {
                        input: path("NextToken")?,
                        output: path("NextToken")?,
                        optional: false,
                    }),
                )?
            },
        ),
        (
            "RemoveAllBackends",
            deferred(
                "Model requires AppId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes all backend environments from your Amplify project.",
            ),
        ),
        (
            "RemoveBackendConfig",
            deferred(
                "Model requires AppId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the AWS resources required to access the Amplify Admin UI.",
            ),
        ),
        (
            "UpdateBackendAPI",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing backend API resource.",
            ),
        ),
        (
            "UpdateBackendAuth",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceConfig, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing backend authentication resource.",
            ),
        ),
        (
            "UpdateBackendConfig",
            deferred(
                "Model requires AppId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the AWS resources required to access the Amplify Admin UI.",
            ),
        ),
        (
            "UpdateBackendJob",
            deferred(
                "Model requires AppId, BackendEnvironmentName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a specific job.",
            ),
        ),
        (
            "UpdateBackendStorage",
            deferred(
                "Model requires AppId, BackendEnvironmentName, ResourceConfig, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing backend storage resource.",
            ),
        ),
    ])
}
