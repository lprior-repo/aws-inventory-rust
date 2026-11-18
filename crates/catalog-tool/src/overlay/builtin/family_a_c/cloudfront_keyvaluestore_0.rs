// Pinned source: .local/aws-models/models/cloudfront-keyvaluestore/service/2022-07-26/cloudfront-keyvaluestore-2022-07-26.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteKey",
            deferred(
                "Model requires KvsARN, Key, IfMatch. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the key value pair specified by the key.",
            ),
        ),
        (
            "DescribeKeyValueStore",
            deferred(
                "Model requires KvsARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns metadata information about Key Value Store.",
            ),
        ),
        (
            "GetKey",
            deferred(
                "Model requires KvsARN, Key. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a key value pair.",
            ),
        ),
        // Model: Returns a list of key value pairs.
        ("ListKeys", denied(DenyReason::Content)),
        (
            "PutKey",
            deferred(
                "Model requires Key, Value, KvsARN, IfMatch. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new key value pair or replaces the value of an existing key.",
            ),
        ),
        (
            "UpdateKeys",
            deferred(
                "Model requires KvsARN, IfMatch. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Puts or Deletes multiple key value pairs in a single, all-or-nothing operation.",
            ),
        ),
    ])
}
