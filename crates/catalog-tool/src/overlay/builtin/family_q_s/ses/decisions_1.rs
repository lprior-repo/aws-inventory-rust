// Pinned operation purposes: .local/aws-models/models/ses/service/2010-12-01/ses-2010-12-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Enables or disables the publishing of reputation metrics for emails sent using a specific configuration set in a given Amazon Web Services Region. Reputation metrics include bounce and complaint rates. These metrics are published to Amazon CloudWatch. By using CloudWatch, you can create alarms when bounce or complaint rates exceed certain thresholds. You can execute this operation no more than onc
    (
        "UpdateConfigurationSetReputationMetricsEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enables or disables email sending for messages sent using a specific configuration set in a given Amazon Web Services Region. You can use this operation in conjunction with Amazon CloudWatch alarms to temporarily pause email sending for a configuration set when the reputation metrics for that configuration set (such as your bounce on complaint rate) exceed certain thresholds. You can execute this
    (
        "UpdateConfigurationSetSendingEnabled",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies an association between a configuration set and a custom domain for open and click event tracking. By default, images and links used for tracking open and click events are hosted on domains operated by Amazon SES. You can configure a subdomain of your own to handle these events. For information about using custom domains, see the Amazon SES Developer Guide .
    (
        "UpdateConfigurationSetTrackingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing custom verification email template. For more information about custom verification email templates, see Using Custom Verification Email Templates in the Amazon SES Developer Guide . You can execute this operation no more than once per second.
    (
        "UpdateCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a receipt rule. For information about managing receipt rules, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.
    ("UpdateReceiptRule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an email template. Email templates enable you to send personalized email to one or more destinations in a single operation. For more information, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.
    ("UpdateTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "VerifyDomainDkim",
        Decision::Deferred(
            "Required selectors Domain are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DkimTokens. Pinned purpose: Returns a set of DKIM tokens for a domain identity. When you execute the VerifyDomainDkim operation, the domain that you specify is added to the list of identities that are associated with your account. This is true even if you haven't already associ",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a domain to the list of identities for your Amazon SES account in the current Amazon Web Services Region and attempts to verify it. For more information about verifying domains, see Verifying Email Addresses and Domains in the Amazon SES Developer Guide. You can execute this operation no more than once per second.
    (
        "VerifyDomainIdentity",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "VerifyEmailAddress",
        Decision::Deferred(
            "Required selectors EmailAddress are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Deprecated. Use the VerifyEmailIdentity operation to verify a new email address.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds an email address to the list of identities for your Amazon SES account in the current Amazon Web Services Region and attempts to verify it. As a result of executing this operation, a verification email is sent to the specified address. You can execute this operation no more than once per second.
    (
        "VerifyEmailIdentity",
        Decision::Denied(DenyReason::Mutating),
    ),
];
