use super::support::{arn, bound, describe, inventory, key, queried, reviewed, scoped, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "eventbridge",
        vec![
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
            queried(
                inventory(
                    "ListArchives",
                    "Archives",
                    key(&["ArchiveName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ArchiveName", "EventSourceArn", "State", "StateReason"],
            )?,
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
            queried(
                bound(
                    describe("DescribeEventBus", "$", arn("Arn")?)?,
                    &[("Name", "ListEventBuses", "Name")],
                )?,
                &["Arn", "Name", "Policy", "Description"],
            )?,
            queried(
                bound(
                    describe("DescribeEventSource", "$", arn("Arn")?)?,
                    &[("Name", "ListEventSources", "Name")],
                )?,
                &["Arn", "Name", "State"],
            )?,
            queried(
                bound(
                    describe("DescribeEndpoint", "$", arn("Arn")?)?,
                    &[("Name", "ListEndpoints", "Name")],
                )?,
                &["Arn", "Name", "EndpointId", "State"],
            )?,
            queried(
                inventory(
                    "ListEventBuses",
                    "EventBuses",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Name"],
            )?,
            queried(
                inventory(
                    "ListEventSources",
                    "EventSources",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Name", "State"],
            )?,
            queried(
                inventory(
                    "ListReplays",
                    "Replays",
                    key(&["ReplayName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ReplayName", "EventSourceArn", "State", "StateReason"],
            )?,
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
            queried(
                bound(
                    describe("DescribeRule", "$", arn("Arn")?)?,
                    &[
                        ("Name", "ListRules", "Name"),
                        ("EventBusName", "ListRules", "EventBusName"),
                    ],
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
