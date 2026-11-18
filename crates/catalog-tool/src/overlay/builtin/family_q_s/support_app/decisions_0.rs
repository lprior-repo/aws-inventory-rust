// Pinned operation purposes: .local/aws-models/models/support-app/service/2021-08-20/support-app-2021-08-20.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Slack channel configuration for your Amazon Web Services account. You can add up to 5 Slack workspaces for your account. You can add up to 20 Slack channels for your account. A Slack channel can have up to 100 Amazon Web Services accounts. This means that only 100 accounts can add the same Slack channel to the Amazon Web Services Support App. We recommend that you only add the accounts t
    (
        "CreateSlackChannelConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an alias for an Amazon Web Services account ID. The alias appears in the Amazon Web Services Support App page of the Amazon Web Services Support Center. The alias also appears in Slack messages from the Amazon Web Services Support App.
    ("DeleteAccountAlias", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Slack channel configuration from your Amazon Web Services account. This operation doesn't delete your Slack channel.
    (
        "DeleteSlackChannelConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Slack workspace configuration from your Amazon Web Services account. This operation doesn't delete your Slack workspace.
    (
        "DeleteSlackWorkspaceConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAccountAlias",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members accountAlias. Pinned request/ownership contract: Retrieves the alias from an Amazon Web Services account ID. The alias appears in the Amazon Web Services Support App page of the Amazon Web Services Support Center. The alias also appears in Slack messages from the Amazon Web Services Support App.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates an individual alias for each Amazon Web Services account ID. The alias appears in the Amazon Web Services Support App page of the Amazon Web Services Support Center. The alias also appears in Slack messages from the Amazon Web Services Support App.
    ("PutAccountAlias", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Registers a Slack workspace for your Amazon Web Services account. To call this API, your account must be part of an organization in Organizations. If you're the management account and you want to register Slack workspaces for your organization, you must complete the following tasks: Sign in to the Amazon Web Services Support Center and authorize the Slack workspaces where you want your organizatio
    (
        "RegisterSlackWorkspaceForOrganization",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration for a Slack channel, such as case update notifications.
    (
        "UpdateSlackChannelConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
];
