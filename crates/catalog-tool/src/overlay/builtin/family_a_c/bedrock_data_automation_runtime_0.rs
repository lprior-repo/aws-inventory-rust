use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetDataAutomationStatus",
            deferred(
                "Model requires invocationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: API used to get data automation status.",
            ),
        ),
        (
            "InvokeDataAutomation",
            deferred(
                "Model requires inputConfiguration, dataAutomationProfileArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sync API: Invoke data automation.",
            ),
        ),
        (
            "InvokeDataAutomationAsync",
            deferred(
                "Model requires inputConfiguration, outputConfiguration, dataAutomationProfileArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Async API: Invoke data automation.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List tags for an Amazon Bedrock Data Automation resource",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceARN, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tag an Amazon Bedrock Data Automation resource",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceARN, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Untag an Amazon Bedrock Data Automation resource",
            ),
        ),
    ])
}
