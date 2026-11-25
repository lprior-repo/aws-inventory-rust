use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CreateSlackChannelConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAccountAlias", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteSlackChannelConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    ("PutAccountAlias", Decision::Denied(DenyReason::Mutating)),
    (
        "RegisterSlackWorkspaceForOrganization",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSlackChannelConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
];
