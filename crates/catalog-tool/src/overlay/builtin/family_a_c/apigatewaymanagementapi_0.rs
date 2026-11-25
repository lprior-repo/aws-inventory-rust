use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteConnection",
            deferred(
                "Model requires ConnectionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete the connection with the provided id.",
            ),
        ),
        (
            "GetConnection",
            deferred(
                "Model requires ConnectionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get information about the connection with the provided id.",
            ),
        ),
        (
            "PostToConnection",
            deferred(
                "Model requires Data, ConnectionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends the provided data to the specified connection.",
            ),
        ),
    ])
}
