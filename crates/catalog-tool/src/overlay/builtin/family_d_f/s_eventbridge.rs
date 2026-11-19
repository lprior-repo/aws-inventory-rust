// Pinned model: .local/aws-models/models/eventbridge/service/2015-10-07/eventbridge-2015-10-07.json
use super::support::{arn, bound, describe, inventory, key, queried, reviewed, scoped, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "eventbridge",
        vec![
            // Retrieves a list of API destination in the account in the current Region.
            queried(
                inventory(
                    "ListApiDestinations",
                    "ApiDestinations",
                    arn("ApiDestinationArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ApiDestinationArn",
                    "ConnectionArn",
                    "Name",
                    "ApiDestinationState",
                ],
            )?,
            // Lists your archives.
            queried(
                inventory(
                    "ListArchives",
                    "Archives",
                    key(&["ArchiveName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ArchiveName", "EventSourceArn", "State", "StateReason"],
            )?,
            // Retrieves a list of connections from the account.
            queried(
                inventory(
                    "ListConnections",
                    "Connections",
                    arn("ConnectionArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ConnectionArn",
                    "Name",
                    "ConnectionState",
                    "StateReason",
                    "AuthorizationType",
                ],
            )?,
            // List the global endpoints associated with this account.
            queried(
                scoped(
                    inventory(
                        "ListEndpoints",
                        "Endpoints",
                        arn("Arn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    Scope::Account,
                ),
                &["Arn", "Name", "EndpointId", "State", "StateReason"],
            )?,
            // Expand DescribeEventBus to include event bus details.
            queried(
                bound(
                    describe("DescribeEventBus", "$", arn("Arn")?)?,
                    &[("Name", "ListEventBuses", "Name")],
                )?,
                &["Arn", "Name", "Policy", "Description"],
            )?,
            // Expand DescribeEventSource to include partner event source details.
            queried(
                bound(
                    describe(
                        "DescribeEventSource",
                        "$",
                        arn("Arn")?,
                    )?,
                    &[("Name", "ListEventSources", "Name")],
                )?,
                &["Arn", "Name", "State"],
            )?,
            // Expand DescribeEndpoint to include global endpoint details.
            queried(
                bound(
                    describe(
                        "DescribeEndpoint",
                        "$",
                        arn("Arn")?,
                    )?,
                    &[("Name", "ListEndpoints", "Name")],
                )?,
                &["Arn", "Name", "EndpointId", "State"],
            )?,
            // Lists all the event buses in your account, including the default event bus, custom event buses, and partner event buses.
            queried(
                inventory(
                    "ListEventBuses",
                    "EventBuses",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Name"],
            )?,
            // You can use this to see all the partner event sources that have been shared with your Amazon Web Services account.
            queried(
                inventory(
                    "ListEventSources",
                    "EventSources",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Name", "State"],
            )?,
            // Lists your replays.
            queried(
                inventory(
                    "ListReplays",
                    "Replays",
                    key(&["ReplayName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ReplayName", "EventSourceArn", "State", "StateReason"],
            )?,
            // Lists your Amazon EventBridge rules.
            queried(
                bound(
                    inventory(
                        "ListRules",
                        "Rules",
                        arn("Arn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &[("EventBusName", "ListEventBuses", "Name")],
                )?,
                &["Arn", "RoleArn", "Name", "State", "EventBusName"],
            )?,
            // Expand DescribeEventRule to include rule details.
            queried(
                bound(
                    describe(
                        "DescribeRule",
                        "$",
                        arn("Arn")?,
                    )?,
                    &[("Name", "ListRules", "Name")],
                )?,
                &["Arn", "Name", "State", "EventPattern", "Description"],
            )?,
        ],
        &[
            super::s_eventbridge_decisions_0::ENTRIES,
            super::s_eventbridge_decisions_1::ENTRIES,
        ],
        &[],
    )
}
