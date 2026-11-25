use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateNotificationRule",
            deferred(
                "Model requires Name, EventTypeIds, Resource, Targets, DetailType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a notification rule for a resource. The rule specifies the events you want notifications about and the targets (such as Amazon Q Developer in chat applications topics or Amazon Q Developer in chat applications clients…",
            ),
        ),
        (
            "DeleteNotificationRule",
            deferred(
                "Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a notification rule for a resource.",
            ),
        ),
        (
            "DeleteTarget",
            deferred(
                "Model requires TargetAddress. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specified target for notifications.",
            ),
        ),
        (
            "DescribeNotificationRule",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("Arn")?,
                    Binding::From {
                        operation: name("ListNotificationRules")?,
                        path: path("Arn")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("Tags")?),
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
            "ListEventTypes",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is EventTypes; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns information about the event types available for configuring notifications.",
            ),
        ),
        (
            "ListNotificationRules",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "NotificationRules",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the tags associated with a notification rule.",
            ),
        ),
        (
            "ListTargets",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Targets; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of the notification rule targets for an Amazon Web Services account.",
            ),
        ),
        (
            "Subscribe",
            deferred(
                "Model requires Arn, Target. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an association between a notification rule and an Amazon Q Developer in chat applications topic or Amazon Q Developer in chat applications client so that the associated target can receive notifications when the events…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires Arn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of provided tags with a notification rule.",
            ),
        ),
        (
            "Unsubscribe",
            deferred(
                "Model requires Arn, TargetAddress. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an association between a notification rule and an Amazon Q Developer in chat applications topic so that subscribers to that topic stop receiving notifications when the events described in the rule are triggered.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires Arn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between one or more provided tags and a notification rule.",
            ),
        ),
        (
            "UpdateNotificationRule",
            deferred(
                "Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a notification rule for a resource. You can change the events that trigger the notification rule, the status of the rule, and the targets that receive the notifications. To add or remove tags for a notification rule, you…",
            ),
        ),
    ])
}
