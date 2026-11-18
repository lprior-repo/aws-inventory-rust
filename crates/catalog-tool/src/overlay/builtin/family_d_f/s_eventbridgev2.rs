// Pinned model: .local/aws-models/models/eventbridgev2/service/2025-05-15/eventbridgev2-2025-05-15.json
use super::support::{arn, bound, describe, fixed_token, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "eventbridgev2",
        vec![
            // Lists the event buses visible to the caller: buses the account owns and buses shared with it through AWS RAM.
            queried(
                inventory(
                    "ListEventBuses",
                    "EventBuses",
                    arn("EventBusArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "EventBusArn",
                    "Name",
                    "State",
                    "StateReason",
                    "EventBusAccountId",
                ],
            )?,
            // Lists EventSources as summaries.
            queried(
                inventory(
                    "ListEventSources",
                    "EventSources",
                    arn("EventSourceArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "EventSourceArn",
                    "EventBusArn",
                    "Name",
                    "Type",
                    "State",
                    "EventSourceAccountId",
                ],
            )?,
            // Lists subscribers as summaries.
            queried(
                inventory(
                    "ListSubscribers",
                    "Subscribers",
                    arn("SubscriberArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "SubscriberArn",
                    "EventBusArn",
                    "Name",
                    "Type",
                    "State",
                    "SubscriberAccountId",
                ],
            )?,
            // DescribeEventBus returns the full configuration of an event bus.
            queried(
                fixed_token(
                    bound(
                        describe("DescribeEventBus", "EventBus", arn("EventBusArn")?)?,
                        &[("EventBusArn", "ListEventBuses", "EventBusArn")],
                    )?,
                    ("NextToken", "NextToken"),
                )?,
                &[
                    "EventBusArn",
                    "Name",
                    "Description",
                    "State",
                    "CreationTime",
                    "LastModifiedTime",
                ],
            )?,
            // DescribeEventSource returns the full configuration of an event source.
            queried(
                fixed_token(
                    bound(
                        describe("DescribeEventSource", "EventSource", arn("EventSourceArn")?)?,
                        &[("EventSourceArn", "ListEventSources", "EventSourceArn")],
                    )?,
                    ("NextToken", "NextToken"),
                )?,
                &[
                    "EventSourceArn",
                    "EventBusArn",
                    "Name",
                    "Description",
                    "State",
                    "CreationTime",
                    "LastModifiedTime",
                ],
            )?,
            // DescribeSubscriber returns the full configuration of a subscriber.
            queried(
                fixed_token(
                    bound(
                        describe("DescribeSubscriber", "Subscriber", arn("SubscriberArn")?)?,
                        &[("SubscriberArn", "ListSubscribers", "SubscriberArn")],
                    )?,
                    ("NextToken", "NextToken"),
                )?,
                &[
                    "SubscriberArn",
                    "EventBusArn",
                    "Name",
                    "Description",
                    "State",
                    "CreationTime",
                    "LastModifiedTime",
                ],
            )?,
        ],
        &[super::s_eventbridgev2_decisions_0::ENTRIES],
        &[],
    )
}
