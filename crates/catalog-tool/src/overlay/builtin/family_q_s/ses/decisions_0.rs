use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CloneReceiptRuleSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateConfigurationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateConfigurationSetTrackingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateReceiptFilter",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateReceiptRule", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateReceiptRuleSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteConfigurationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteConfigurationSetTrackingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteIdentity", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteIdentityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteReceiptFilter",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteReceiptRule", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteReceiptRuleSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteVerifiedEmailAddress",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeActiveReceiptRuleSet",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Metadata, Rules. Pinned request/ownership contract: Returns the metadata and receipt rules for the receipt rule set that is currently active. For information about setting up receipt rule sets, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.",
        ),
    ),
    (
        "DescribeConfigurationSet",
        Decision::Deferred(
            "Required selectors ConfigurationSetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationSet, EventDestinations, TrackingOptions, DeliveryOptions, ReputationOptions. Pinned purpose: Returns the details of the specified configuration set. For information about using configuration sets, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.",
        ),
    ),
    (
        "DescribeReceiptRule",
        Decision::Deferred(
            "Required selectors RuleSetName, RuleName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Rule. Pinned purpose: Returns the details of the specified receipt rule. For information about setting up receipt rules, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.",
        ),
    ),
    (
        "DescribeReceiptRuleSet",
        Decision::Deferred(
            "Required selectors RuleSetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Metadata, Rules. Pinned purpose: Returns the details of the specified receipt rule set. For information about managing receipt rule sets, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.",
        ),
    ),
    (
        "GetAccountSendingEnabled",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Enabled. Pinned request/ownership contract: Returns the email sending status of the Amazon SES account for the current Region. You can execute this operation no more than once per second.",
        ),
    ),
    (
        "GetCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetIdentityDkimAttributes",
        Decision::Deferred(
            "Required selectors Identities are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DkimAttributes. Pinned purpose: Returns the current status of Easy DKIM signing for an entity. For domain name identities, this operation also returns the DKIM tokens that are required for Easy DKIM signing, and whether Amazon SES has successfully verified that these tokens have be",
        ),
    ),
    (
        "GetIdentityMailFromDomainAttributes",
        Decision::Deferred(
            "Required selectors Identities are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MailFromDomainAttributes. Pinned purpose: Returns the custom MAIL FROM attributes for a list of identities (email addresses : domains). This operation is throttled at one request per second and can only get custom MAIL FROM attributes for up to 100 identities at a time.",
        ),
    ),
    (
        "GetIdentityNotificationAttributes",
        Decision::Deferred(
            "Required selectors Identities are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NotificationAttributes. Pinned purpose: Given a list of verified identities (email addresses and/or domains), returns a structure describing identity notification attributes. This operation is throttled at one request per second and can only get notification attributes for up to 100 identi",
        ),
    ),
    (
        "GetIdentityPolicies",
        Decision::Deferred(
            "Required selectors Identity, PolicyNames are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policies. Pinned purpose: Returns the requested sending authorization policies for the given identity (an email address or a domain). The policies are returned as a map of policy names to policy contents. You can retrieve a maximum of 20 policies at a time. This operation is ",
        ),
    ),
    (
        "GetIdentityVerificationAttributes",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetSendQuota",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Max24HourSend, MaxSendRate, SentLast24Hours. Pinned request/ownership contract: Provides the sending limits for the Amazon SES account. You can execute this operation no more than once per second.",
        ),
    ),
    (
        "GetSendStatistics",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members SendDataPoints. Pinned request/ownership contract: Provides sending statistics for the current Amazon Web Services Region. The result is a list of data points, representing the last two weeks of sending activity. Each data point in the list contains statistics for a 15-minute period of time. You can execute this operation no more than once per second.",
        ),
    ),
    ("GetTemplate", Decision::Denied(DenyReason::Content)),
    (
        "ListIdentityPolicies",
        Decision::Deferred(
            "Required selectors Identity are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PolicyNames. Pinned purpose: Returns a list of sending authorization policies that are attached to the given identity (an email address or a domain). This operation returns only a list. To get the actual policy content, use GetIdentityPolicies . This operation is for the identit",
        ),
    ),
    (
        "ListVerifiedEmailAddresses",
        Decision::Deferred(
            "Deprecated verified-email-only subset of ListIdentities; ListIdentities preserves email and domain identity inventory without silently dropping domain identities.",
        ),
    ),
    (
        "PutConfigurationSetDeliveryOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutIdentityPolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "ReorderReceiptRuleSet",
        Decision::Deferred(
            "Required selectors RuleSetName, RuleNames are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Reorders the receipt rules within a receipt rule set. All of the rules in the rule set must be represented in this request. That is, it is error if the reorder request doesn't explicitly position all of the rules. For information about managing recei",
        ),
    ),
    ("SendBounce", Decision::Denied(DenyReason::Mutating)),
    (
        "SendBulkTemplatedEmail",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SendCustomVerificationEmail",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("SendEmail", Decision::Denied(DenyReason::Mutating)),
    ("SendRawEmail", Decision::Denied(DenyReason::Mutating)),
    ("SendTemplatedEmail", Decision::Denied(DenyReason::Mutating)),
    (
        "SetActiveReceiptRuleSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetIdentityDkimEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetIdentityFeedbackForwardingEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetIdentityHeadersInNotificationsEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetIdentityMailFromDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetIdentityNotificationTopic",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SetReceiptRulePosition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TestRenderTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAccountSendingEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
];
