// Pinned source: .local/aws-models/models/appconfigdata/service/2021-11-11/appconfigdata-2021-11-11.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetLatestConfiguration",
            deferred(
                "Model requires ConfigurationToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the latest deployed configuration. This API may return empty configuration data if the client already has the latest version. For more information about this API action and to view example CLI commands that show how to…",
            ),
        ),
        (
            "StartConfigurationSession",
            deferred(
                "Model requires ApplicationIdentifier, EnvironmentIdentifier, ConfigurationProfileIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a configuration session used to retrieve a deployed configuration. For more information about this API action and to view example CLI commands that show how to use it with the GetLatestConfiguration API action, see…",
            ),
        ),
    ])
}
