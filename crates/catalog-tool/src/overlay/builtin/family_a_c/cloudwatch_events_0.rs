use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ActivateEventSource",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Activates a partner event source that has been deactivated. Once activated, your matching event bus will start receiving events from the event source.",
            ),
        ),
        (
            "CancelReplay",
            deferred(
                "Model requires ReplayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels the specified replay.",
            ),
        ),
        (
            "CreateApiDestination",
            deferred(
                "Model requires Name, ConnectionArn, InvocationEndpoint, HttpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an API destination, which is an HTTP invocation endpoint configured as a target for events.",
            ),
        ),
        (
            "CreateArchive",
            deferred(
                "Model requires ArchiveName, EventSourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an archive of events with the specified settings. When you create an archive, incoming events might not immediately start being sent to the archive. Allow a short period of time for changes to take effect. If you do not…",
            ),
        ),
        (
            "CreateConnection",
            deferred(
                "Model requires Name, AuthorizationType, AuthParameters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a connection. A connection defines the authorization type and credentials to use for authorization with an API destination HTTP endpoint.",
            ),
        ),
        (
            "CreateEventBus",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new event bus within your account. This can be a custom event bus which you can use to receive events from your custom applications and services, or it can be a partner event bus which can be matched to a partner event…",
            ),
        ),
        (
            "CreatePartnerEventSource",
            deferred(
                "Model requires Name, Account. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Called by an SaaS partner to create a partner event source. This operation is not used by Amazon Web Services customers. Each partner event source can be used by one Amazon Web Services account to create a matching partner event…",
            ),
        ),
        (
            "DeactivateEventSource",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: You can use this operation to temporarily stop receiving events from the specified partner event source. The matching event bus is not deleted. When you deactivate a partner event source, the source goes into PENDING state. If it…",
            ),
        ),
        (
            "DeauthorizeConnection",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes all authorization parameters from the connection. This lets you remove the secret from the connection so you can reuse it without having to create a new connection.",
            ),
        ),
        (
            "DeleteApiDestination",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified API destination.",
            ),
        ),
        (
            "DeleteArchive",
            deferred(
                "Model requires ArchiveName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified archive.",
            ),
        ),
        (
            "DeleteConnection",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a connection.",
            ),
        ),
        (
            "DeleteEventBus",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified custom event bus or partner event bus. All rules associated with this event bus need to be deleted. You can't delete your account's default event bus.",
            ),
        ),
        (
            "DeletePartnerEventSource",
            deferred(
                "Model requires Name, Account. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation is used by SaaS partners to delete a partner event source. This operation is not used by Amazon Web Services customers. When you delete an event source, the status of the corresponding partner event bus in the…",
            ),
        ),
        (
            "DeleteRule",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified rule. Before you can delete the rule, you must remove all targets, using RemoveTargets . When you delete a rule, incoming events might continue to match to the deleted rule. Allow a short period of time for…",
            ),
        ),
        (
            "DescribeApiDestination",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("Name")?,
                    Binding::From {
                        operation: name("ListApiDestinations")?,
                        path: path("Name")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ApiDestinationArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("ApiDestinationArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeArchive",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ArchiveName")?,
                    Binding::From {
                        operation: name("ListArchives")?,
                        path: path("ArchiveName")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ArchiveName")?, path("ArchiveArn")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("ArchiveName")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        ("DescribeConnection", denied(DenyReason::Secret)),
        (
            "DescribeEventBus",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("Name")?,
                    Binding::From {
                        operation: name("ListEventBuses")?,
                        path: path("Name")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Arn")?],
                ..read(
                    "$",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeEventSource",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation lists details about a partner event source that is shared with your account.",
            ),
        ),
        (
            "DescribePartnerEventSource",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An SaaS partner can use this operation to list details about a partner event source that they have created. Amazon Web Services customers do not use this operation. Instead, Amazon Web Services customers can use…",
            ),
        ),
        (
            "DescribeReplay",
            deferred(
                "Model requires ReplayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a replay. Use DescribeReplay to determine the progress of a running replay. A replay processes events to replay based on the time in the event, and replays them using 1 minute intervals. If you use…",
            ),
        ),
        (
            "DescribeRule",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified rule. DescribeRule does not list the targets of a rule. To see the targets associated with a rule, use ListTargetsByRule .",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DisableRule",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disables the specified rule. A disabled rule won't match any events, and won't self-trigger if it has a schedule expression. When you disable a rule, incoming events might continue to match to the disabled rule. Allow a short…",
            ),
        ),
        (
            "EnableRule",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables the specified rule. If the rule does not exist, the operation fails. When you enable a rule, incoming events might not immediately start matching to a newly enabled rule. Allow a short period of time for changes to take…",
            ),
        ),
        (
            "ListApiDestinations",
            OperationOverlay {
                queryable: vec![path("ApiDestinationArn")?],
                ..read(
                    "ApiDestinations",
                    Identity::Arn(path("ApiDestinationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListArchives",
            OperationOverlay {
                queryable: vec![path("ArchiveName")?, path("EventSourceArn")?],
                ..read(
                    "Archives",
                    Identity::Key(vec![path("ArchiveName")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListConnections",
            OperationOverlay {
                queryable: vec![path("ConnectionArn")?],
                ..read(
                    "Connections",
                    Identity::Arn(path("ConnectionArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListEventBuses",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "EventBuses",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListEventSources",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "EventSources",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListPartnerEventSourceAccounts",
            deferred(
                "Model requires EventSourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An SaaS partner can use this operation to display the Amazon Web Services account ID that a particular partner event source name is associated with. This operation is not used by Amazon Web Services customers.",
            ),
        ),
        (
            "ListPartnerEventSources",
            deferred(
                "Model requires NamePrefix. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An SaaS partner can use this operation to list all the partner event source names that they have created. This operation is not used by Amazon Web Services customers.",
            ),
        ),
        (
            "ListReplays",
            OperationOverlay {
                queryable: vec![path("ReplayName")?],
                ..read(
                    "Replays",
                    Identity::Key(vec![path("ReplayName")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListRuleNamesByTarget",
            deferred(
                "Model requires TargetArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the rules for the specified target. You can see which of the rules in Amazon EventBridge can invoke a specific target in your account.",
            ),
        ),
        (
            "ListRules",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("EventBusName")?,
                    Binding::From {
                        operation: name("ListEventBuses")?,
                        path: path("Name")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Arn")?],
                ..read(
                    "Rules",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Displays the tags associated with an EventBridge resource. In EventBridge, rules and event buses can be tagged.",
            ),
        ),
        (
            "ListTargetsByRule",
            deferred(
                "Model requires Rule. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the targets assigned to the specified rule.",
            ),
        ),
        (
            "PutEvents",
            deferred(
                "Model requires Entries. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends custom events to Amazon EventBridge so that they can be matched to rules.",
            ),
        ),
        (
            "PutPartnerEvents",
            deferred(
                "Model requires Entries. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is used by SaaS partners to write events to a customer's partner event bus. Amazon Web Services customers do not use this operation.",
            ),
        ),
        (
            "PutPermission",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Running PutPermission permits the specified Amazon Web Services account or Amazon Web Services organization to put events to the specified event bus . Amazon EventBridge (CloudWatch Events) rules in your account are triggered by…",
            ),
        ),
        (
            "PutRule",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates the specified rule. Rules are enabled by default, or based on value of the state. You can disable a rule using DisableRule . A single rule watches for events from a single event bus. Events generated by Amazon…",
            ),
        ),
        (
            "PutTargets",
            deferred(
                "Model requires Rule, Targets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds the specified targets to the specified rule, or updates the targets if they are already associated with the rule. Targets are the resources that are invoked when a rule is triggered. You can configure the following as…",
            ),
        ),
        (
            "RemovePermission",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Revokes the permission of another Amazon Web Services account to be able to put events to the specified event bus. Specify the account to revoke by the StatementId value that you associated with the account when you granted it…",
            ),
        ),
        (
            "RemoveTargets",
            deferred(
                "Model requires Rule, Ids. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified targets from the specified rule. When the rule is triggered, those targets are no longer be invoked. When you remove a target, when the associated rule triggers, removed targets might continue to be invoked.…",
            ),
        ),
        (
            "StartReplay",
            deferred(
                "Model requires ReplayName, EventSourceArn, EventStartTime, EventEndTime, Destination. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts the specified replay. Events are not necessarily replayed in the exact same order that they were added to the archive. A replay processes events to replay based on the time in the event, and replays them using 1 minute…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns one or more tags (key-value pairs) to the specified EventBridge resource. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access…",
            ),
        ),
        (
            "TestEventPattern",
            deferred(
                "Model requires EventPattern, Event. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tests whether the specified event pattern matches the provided event. Most services in Amazon Web Services treat : or / as the same character in Amazon Resource Names (ARNs). However, EventBridge uses an exact match in event…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from the specified EventBridge resource. In Amazon EventBridge (CloudWatch Events), rules and event buses can be tagged.",
            ),
        ),
        (
            "UpdateApiDestination",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an API destination.",
            ),
        ),
        (
            "UpdateArchive",
            deferred(
                "Model requires ArchiveName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified archive.",
            ),
        ),
        (
            "UpdateConnection",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates settings for a connection.",
            ),
        ),
    ])
}
