pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateDeliveryStream",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Creates a Firehose stream.",
    ),
    (
        "DeleteDeliveryStream",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Deletes a Firehose stream and its data.",
    ),
    (
        "ListTagsForDeliveryStream",
        "Stream enumeration is supported, but the stream-bound tag request and tag endpoint have not been reviewed. Pinned operation: Lists the tags for the specified Firehose stream.",
    ),
    (
        "StartDeliveryStreamEncryption",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Enables server-side encryption (SSE) for the Firehose stream.",
    ),
    (
        "StopDeliveryStreamEncryption",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Disables server-side encryption (SSE) for the Firehose stream.",
    ),
    (
        "TagDeliveryStream",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Adds or updates tags for the specified Firehose stream.",
    ),
    (
        "UntagDeliveryStream",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Removes tags from the specified Firehose stream.",
    ),
    (
        "UpdateDestination",
        "State-changing operation; no inventory read capability is approved. Pinned operation: Updates the specified destination of the specified Firehose stream.",
    ),
];
