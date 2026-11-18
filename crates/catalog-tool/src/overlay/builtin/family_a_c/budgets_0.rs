// Pinned source: .local/aws-models/models/budgets/service/2016-10-20/budgets-2016-10-20.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateBudget",
            deferred(
                "Model requires AccountId, Budget. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a budget and, if included, notifications and subscribers. Only one of BudgetLimit or PlannedBudgetLimits can be present in the syntax at one time. Use the syntax that matches your use case. The Request Syntax section…",
            ),
        ),
        (
            "CreateBudgetAction",
            deferred(
                "Model requires AccountId, BudgetName, NotificationType, ActionType, ActionThreshold, Definition, ExecutionRoleArn, ApprovalModel, Subscribers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a budget action.",
            ),
        ),
        (
            "CreateNotification",
            deferred(
                "Model requires AccountId, BudgetName, Notification, Subscribers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a notification. You must create the budget before you create the associated notification.",
            ),
        ),
        (
            "CreateSubscriber",
            deferred(
                "Model requires AccountId, BudgetName, Notification, Subscriber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a subscriber. You must create the associated budget and notification before you create the subscriber.",
            ),
        ),
        (
            "DeleteBudget",
            deferred(
                "Model requires AccountId, BudgetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a budget. You can delete your budget at any time. Deleting a budget also deletes the notifications and subscribers that are associated with that budget.",
            ),
        ),
        (
            "DeleteBudgetAction",
            deferred(
                "Model requires AccountId, BudgetName, ActionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a budget action.",
            ),
        ),
        (
            "DeleteNotification",
            deferred(
                "Model requires AccountId, BudgetName, Notification. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a notification. Deleting a notification also deletes the subscribers that are associated with the notification.",
            ),
        ),
        (
            "DeleteSubscriber",
            deferred(
                "Model requires AccountId, BudgetName, Notification, Subscriber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a subscriber. Deleting the last subscriber to a notification also deletes the notification.",
            ),
        ),
        (
            "DescribeBudget",
            deferred(
                "Model requires AccountId, BudgetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a budget. The Request Syntax section shows the BudgetLimit syntax. For PlannedBudgetLimits , see the Examples section.",
            ),
        ),
        (
            "DescribeBudgetAction",
            deferred(
                "Model requires AccountId, BudgetName, ActionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a budget action detail.",
            ),
        ),
        (
            "DescribeBudgetActionHistories",
            deferred(
                "Model requires AccountId, BudgetName, ActionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a budget action history detail.",
            ),
        ),
        (
            "DescribeBudgetActionsForAccount",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes all of the budget actions for an account.",
            ),
        ),
        (
            "DescribeBudgetActionsForBudget",
            deferred(
                "Model requires AccountId, BudgetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes all of the budget actions for a budget.",
            ),
        ),
        (
            "DescribeBudgetNotificationsForAccount",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the budget names and notifications that are associated with an account.",
            ),
        ),
        (
            "DescribeBudgetPerformanceHistory",
            deferred(
                "Model requires AccountId, BudgetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the history for DAILY , MONTHLY , and QUARTERLY budgets. Budget history isn't available for ANNUAL budgets.",
            ),
        ),
        // Model: Lists the budgets that are associated with an account. The Request Syntax section shows the BudgetLimit syntax. For PlannedBudgetLimits , see the Examples section.
        (
            "DescribeBudgets",
            OperationOverlay {
                bind: [(path("AccountId")?, Binding::Account)]
                    .into_iter()
                    .collect(),
                queryable: vec![path("BudgetName")?],
                ..read(
                    "Budgets",
                    Identity::Key(vec![path("BudgetName")?]),
                    Scope::Account,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeNotificationsForBudget",
            deferred(
                "Model requires AccountId, BudgetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the notifications that are associated with a budget.",
            ),
        ),
        (
            "DescribeSubscribersForNotification",
            deferred(
                "Model requires AccountId, BudgetName, Notification. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the subscribers that are associated with a notification.",
            ),
        ),
        (
            "ExecuteBudgetAction",
            deferred(
                "Model requires AccountId, BudgetName, ActionId, ExecutionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Executes a budget action.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists tags associated with a budget or budget action resource.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, ResourceTags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates tags for a budget or budget action resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, ResourceTagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes tags associated with a budget or budget action resource.",
            ),
        ),
        (
            "UpdateBudget",
            deferred(
                "Model requires AccountId, NewBudget. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a budget. You can change every part of a budget except for the budgetName and the calculatedSpend . When you modify a budget, the calculatedSpend drops to zero until Amazon Web Services has new usage data to use for…",
            ),
        ),
        (
            "UpdateBudgetAction",
            deferred(
                "Model requires AccountId, BudgetName, ActionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a budget action.",
            ),
        ),
        (
            "UpdateNotification",
            deferred(
                "Model requires AccountId, BudgetName, OldNotification, NewNotification. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a notification.",
            ),
        ),
        (
            "UpdateSubscriber",
            deferred(
                "Model requires AccountId, BudgetName, Notification, OldSubscriber, NewSubscriber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a subscriber.",
            ),
        ),
    ])
}
