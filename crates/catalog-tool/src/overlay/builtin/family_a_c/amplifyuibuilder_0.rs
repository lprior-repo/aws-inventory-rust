// Pinned source: .local/aws-models/models/amplifyuibuilder/service/2021-08-11/amplifyuibuilder-2021-08-11.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateComponent",
            deferred(
                "Model requires appId, environmentName, componentToCreate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new component for an Amplify app.",
            ),
        ),
        (
            "CreateForm",
            deferred(
                "Model requires appId, environmentName, formToCreate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new form for an Amplify app.",
            ),
        ),
        (
            "CreateTheme",
            deferred(
                "Model requires appId, environmentName, themeToCreate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a theme to apply to the components in an Amplify app.",
            ),
        ),
        (
            "DeleteComponent",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a component from an Amplify app.",
            ),
        ),
        (
            "DeleteForm",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a form from an Amplify app.",
            ),
        ),
        (
            "DeleteTheme",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a theme from an Amplify app.",
            ),
        ),
        (
            "ExchangeCodeForToken",
            deferred(
                "Model requires provider, request. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is for internal use. Amplify uses this action to exchange an access code for a token.",
            ),
        ),
        (
            "ExportComponents",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports component configurations to code that is ready to integrate into an Amplify app.",
            ),
        ),
        (
            "ExportForms",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports form configurations to code that is ready to integrate into an Amplify app.",
            ),
        ),
        (
            "ExportThemes",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports theme configurations to code that is ready to integrate into an Amplify app.",
            ),
        ),
        (
            "GetCodegenJob",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an existing code generation job.",
            ),
        ),
        (
            "GetComponent",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an existing component for an Amplify app.",
            ),
        ),
        (
            "GetForm",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an existing form for an Amplify app.",
            ),
        ),
        (
            "GetMetadata",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns existing metadata for an Amplify app.",
            ),
        ),
        (
            "GetTheme",
            deferred(
                "Model requires appId, environmentName, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an existing theme for an Amplify app.",
            ),
        ),
        (
            "ListCodegenJobs",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of code generation jobs for a specified Amplify app and backend environment.",
            ),
        ),
        (
            "ListComponents",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of components for a specified Amplify app and backend environment.",
            ),
        ),
        (
            "ListForms",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of forms for a specified Amplify app and backend environment.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags for a specified Amazon Resource Name (ARN).",
            ),
        ),
        (
            "ListThemes",
            deferred(
                "Model requires appId, environmentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of themes for a specified Amplify app and backend environment.",
            ),
        ),
        (
            "PutMetadataFlag",
            deferred(
                "Model requires appId, environmentName, featureName, body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stores the metadata information about a feature on a form.",
            ),
        ),
        (
            "RefreshToken",
            deferred(
                "Model requires provider, refreshTokenBody. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is for internal use. Amplify uses this action to refresh a previously issued access token that might have expired.",
            ),
        ),
        (
            "StartCodegenJob",
            deferred(
                "Model requires appId, environmentName, codegenJobToCreate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a code generation job for a specified Amplify app and backend environment.",
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
            "UpdateComponent",
            deferred(
                "Model requires appId, environmentName, id, updatedComponent. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing component.",
            ),
        ),
        (
            "UpdateForm",
            deferred(
                "Model requires appId, environmentName, id, updatedForm. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing form.",
            ),
        ),
        (
            "UpdateTheme",
            deferred(
                "Model requires appId, environmentName, id, updatedTheme. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing theme.",
            ),
        ),
    ])
}
