pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ActivateEventSource",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Activates a partner event source that has been deactivated.",
    ),
    (
        "CancelReplay",
        "Requires ReplayName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Cancels the specified replay.",
    ),
    (
        "CreateApiDestination",
        "Requires Name, ConnectionArn, InvocationEndpoint, HttpMethod; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an API destination, which is an HTTP invocation endpoint configured as a target for events.",
    ),
    (
        "CreateArchive",
        "Requires ArchiveName, EventSourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an archive of events with the specified settings.",
    ),
    (
        "CreateConnection",
        "Requires Name, AuthorizationType, AuthParameters; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a connection.",
    ),
    (
        "CreateEndpoint",
        "Requires Name, RoutingConfig, EventBuses; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a global endpoint.",
    ),
    (
        "CreateEventBus",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new event bus within your account.",
    ),
    (
        "CreatePartnerEventSource",
        "Requires Name, Account; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Called by an SaaS partner to create a partner event source.",
    ),
    (
        "DeactivateEventSource",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: You can use this operation to temporarily stop receiving events from the specified partner event source.",
    ),
    (
        "DeauthorizeConnection",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes all authorization parameters from the connection.",
    ),
    (
        "DeleteApiDestination",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified API destination.",
    ),
    (
        "DeleteArchive",
        "Requires ArchiveName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified archive.",
    ),
    (
        "DeleteConnection",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a connection.",
    ),
    (
        "DeleteEndpoint",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Delete an existing global endpoint.",
    ),
    (
        "DeleteEventBus",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified custom event bus or partner event bus.",
    ),
    (
        "DeletePartnerEventSource",
        "Requires Name, Account; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation is used by SaaS partners to delete a partner event source.",
    ),
    (
        "DeleteRule",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified rule.",
    ),
    (
        "DescribeApiDestination",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves details about an API destination.",
    ),
    (
        "DescribeArchive",
        "Requires ArchiveName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves details about an archive.",
    ),
    (
        "DescribeConnection",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves details about a connection.",
    ),
    (
        "DescribePartnerEventSource",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: An SaaS partner can use this operation to list details about a partner event source that they have created.",
    ),
    (
        "DescribeReplay",
        "Requires ReplayName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves details about a replay.",
    ),
    (
        "DisableRule",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disables the specified rule.",
    ),
    (
        "EnableRule",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Enables the specified rule.",
    ),
    (
        "ListPartnerEventSourceAccounts",
        "Requires EventSourceName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: An SaaS partner can use this operation to display the Amazon Web Services account ID that a particular partner event source name is associated with.",
    ),
    (
        "ListPartnerEventSources",
        "Requires NamePrefix; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: An SaaS partner can use this operation to list all the partner event source names that they have created.",
    ),
    (
        "ListRuleNamesByTarget",
        "Requires TargetArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the rules for the specified target.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceARN; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays the tags associated with an EventBridge resource.",
    ),
    (
        "ListTargetsByRule",
        "Targets.Id is unique only within its rule; response lacks the rule and bus identity. Runtime cannot include bound input in selected-item identity; Target.Arn alone is not association identity. Pinned operation: Lists the targets assigned to the specified rule.",
    ),
    (
        "PutEvents",
        "Requires Entries; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Sends custom events to Amazon EventBridge so that they can be matched to rules.",
    ),
    (
        "PutPartnerEvents",
        "Requires Entries; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This is used by SaaS partners to write events to a customer's partner event bus.",
    ),
];
