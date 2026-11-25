pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "PutPermission",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Running PutPermission permits the specified Amazon Web Services account or Amazon Web Services organization to put events to the specified event bus .",
    ),
    (
        "PutRule",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates the specified rule.",
    ),
    (
        "PutTargets",
        "Requires Rule, Targets; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds the specified targets to the specified rule, or updates the targets if they are already associated with the rule.",
    ),
    (
        "RemovePermission",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Revokes the permission of another Amazon Web Services account to be able to put events to the specified event bus.",
    ),
    (
        "RemoveTargets",
        "Requires Rule, Ids; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified targets from the specified rule.",
    ),
    (
        "StartReplay",
        "Requires ReplayName, EventSourceArn, EventStartTime, EventEndTime, Destination; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts the specified replay.",
    ),
    (
        "TagResource",
        "Requires ResourceARN, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Assigns one or more tags (key-value pairs) to the specified EventBridge resource.",
    ),
    (
        "TestEventPattern",
        "Requires EventPattern, Event; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Tests whether the specified event pattern matches the provided event.",
    ),
    (
        "UntagResource",
        "Requires ResourceARN, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes one or more tags from the specified EventBridge resource.",
    ),
    (
        "UpdateApiDestination",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an API destination.",
    ),
    (
        "UpdateArchive",
        "Requires ArchiveName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified archive.",
    ),
    (
        "UpdateConnection",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates settings for a connection.",
    ),
    (
        "UpdateEndpoint",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Update an existing endpoint.",
    ),
    (
        "UpdateEventBus",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Updates the specified event bus.",
    ),
];
