use super::support::{arn, bound, describe, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "eventbridgev2",
        vec![
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
            queried(
                bound(
                    describe("DescribeEventBus", "$", arn("EventBusArn")?)?,
                    &[("EventBusArn", "ListEventBuses", "EventBusArn")],
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
            queried(
                bound(
                    describe("DescribeEventSource", "$", arn("EventSourceArn")?)?,
                    &[("EventSourceArn", "ListEventSources", "EventSourceArn")],
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
        ],
        &[super::s_eventbridgev2_decisions_0::ENTRIES],
        &[("DescribeSubscriber", crate::classes::DenyReason::Secret)],
    )
}
