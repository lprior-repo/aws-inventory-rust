pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AssociateFeed",
        "Requires id, associatedResourceName, outputs; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates a resource with the feed.",
    ),
    (
        "CreateDictionary",
        "Requires name, language; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a custom dictionary for improving transcription accuracy.",
    ),
    (
        "CreateFeed",
        "Requires name, outputs; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a feed.",
    ),
    (
        "CreateScheduleAction",
        "Requires id, actionName, startConfiguration, operations; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a schedule action on a feed.",
    ),
    (
        "DeleteDictionary",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified dictionary.",
    ),
    (
        "DeleteFeed",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified feed.",
    ),
    (
        "DeleteFeedPolicy",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the resource-based policy attached to the specified feed.",
    ),
    (
        "DeleteScheduleAction",
        "Requires id, actionName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a schedule action from a feed.",
    ),
    (
        "DisassociateFeed",
        "Requires id, associatedResourceName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Releases the resource (the source media) that is associated with this feed.",
    ),
    (
        "ExportDictionaryEntries",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Exports the entries from the specified dictionary.",
    ),
    (
        "GetFeed",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about the specified feed.",
    ),
    (
        "GetFeedPolicy",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the resource-based policy attached to the specified feed.",
    ),
    (
        "GetFixture",
        "Requires fixtureId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about the specified fixture (a sports event, such as a specific basketball game).",
    ),
    (
        "ListScheduleActions",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays a list of the schedule actions on a feed, including the status of each action and, for a failed action, information about why it failed.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: List all tags that are on an Elemental Inference resource in the current region.",
    ),
    (
        "PutFeedPolicy",
        "Requires id, policy; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Attaches or replaces a resource-based policy on the specified feed.",
    ),
    (
        "SearchFixtures",
        "Requires sport, startDate; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Searches for the fixtures (sports events, such as a specific basketball game) that are available for a sport in a date window.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates the specified tags to the resource identified by the specified resourceArn in the current region.",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes specified tags from the specified resource in the current region.",
    ),
    (
        "UpdateDictionary",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified dictionary.",
    ),
    (
        "UpdateFeed",
        "Requires name, id, outputs; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the name and/or outputs in a feed.",
    ),
];
