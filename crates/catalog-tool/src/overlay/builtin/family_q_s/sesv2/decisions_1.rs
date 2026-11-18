// Pinned operation purposes: .local/aws-models/models/sesv2/service/2019-09-27/sesv2-2019-09-27.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "GetTenant",
        Decision::Deferred(
            "Required selectors TenantName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Tenant. Pinned purpose: Get information about a specific tenant, including the tenant's name, ID, ARN, creation timestamp, tags, sending status, and suppression attributes.",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Lists the contacts present in a specific contact list.
    ("ListContacts", Decision::Denied(DenyReason::Content)),
    (
        "ListDomainDeliverabilityCampaigns",
        Decision::Deferred(
            "Required selectors StartDate, EndDate, SubscribedDomain are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainDeliverabilityCampaigns, NextToken. Pinned purpose: Retrieve deliverability data for all the campaigns that used a specific domain to send email during a specified time range. This data is available for a domain only if you enabled the Deliverability dashboard for the domain.",
        ),
    ),
    (
        "ListResourceTenants",
        Decision::Deferred(
            "Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceTenants, NextToken. Pinned purpose: List all tenants associated with a specific resource. This operation returns a list of tenants that are associated with the specified resource. This is useful for understanding which tenants are currently using a particular resource such as an email ",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Retrieves a list of email addresses that are on the suppression list for your account or for a specific tenant. To target a tenant's suppression list, specify the TenantName parameter. If you omit TenantName , the operation targets the account-level suppression list.
    (
        "ListSuppressedDestinations",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Retrieve a list of the tags (keys and values) that are associated with a specified resource. A tag is a label that you optionally define and associate with a resource. Each tag consists of a required tag key and an optional associated tag value . A tag key is a general label that acts as a category ",
        ),
    ),
    (
        "ListTenantResources",
        Decision::Deferred(
            "Required selectors TenantName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TenantResources, NextToken. Pinned purpose: List all resources associated with a specific tenant. This operation returns a list of resources (email identities, configuration sets, or email templates) that are associated with the specified tenant. You can optionally filter the results by resour",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable or disable the automatic warm-up feature for dedicated IP addresses.
    (
        "PutAccountDedicatedIpWarmupAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update your Amazon SES account details.
    ("PutAccountDetails", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Set the pricing plan for your Amazon SES account.
    (
        "PutAccountPricingAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable or disable the ability of your account to send email.
    (
        "PutAccountSendingAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Change the settings for the account-level suppression list.
    (
        "PutAccountSuppressionAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update your Amazon SES account VDM attributes. You can execute this operation no more than once per second.
    (
        "PutAccountVdmAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associate the configuration set with a MailManager archive. When you send email using the SendEmail or SendBulkEmail operations the message as it will be given to the receiving SMTP server will be archived, along with the recipient information.
    (
        "PutConfigurationSetArchivingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associate a configuration set with a dedicated IP pool. You can use dedicated IP pools to create groups of dedicated IP addresses for sending specific types of email.
    (
        "PutConfigurationSetDeliveryOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable or disable collection of reputation metrics for emails that you send using a particular configuration set in a specific Amazon Web Services Region.
    (
        "PutConfigurationSetReputationOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable or disable email sending for messages that use a particular configuration set in a specific Amazon Web Services Region.
    (
        "PutConfigurationSetSendingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Specify the suppression list preferences for a configuration set. You can also use this operation to specify a SuppressionScope to override the suppression scope of the tenant or account for emails sent using this configuration set.
    (
        "PutConfigurationSetSuppressionOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Specify a custom domain to use for open and click tracking elements in email that you send.
    (
        "PutConfigurationSetTrackingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Specify VDM preferences for email that you send using the configuration set. You can execute this operation no more than once per second.
    (
        "PutConfigurationSetVdmOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Move a dedicated IP address to an existing dedicated IP pool. The dedicated IP address that you specify must already exist, and must be associated with your Amazon Web Services account. The dedicated IP pool you specify must already exist. You can create a new pool by using the CreateDedicatedIpPool operation.
    (
        "PutDedicatedIpInPool",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to convert a dedicated IP pool to a different scaling mode. MANAGED pools cannot be converted to STANDARD scaling mode.
    (
        "PutDedicatedIpPoolScalingAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: PutDedicatedIpWarmupAttributes
    (
        "PutDedicatedIpWarmupAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable or disable the Deliverability dashboard. When you enable the Deliverability dashboard, you gain access to reputation, deliverability, and other metrics for the domains that you use to send email. You also gain the ability to perform predictive inbox placement tests. When you use the Deliverability dashboard, you pay a monthly subscription charge, in addition to any other fees that you accru
    (
        "PutDeliverabilityDashboardOption",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to associate a configuration set with an email identity.
    (
        "PutEmailIdentityConfigurationSetAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to enable or disable DKIM authentication for an email identity.
    (
        "PutEmailIdentityDkimAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to configure or change the DKIM authentication settings for an email domain identity. You can use this operation to do any of the following: Update the signing attributes for an identity that uses Bring Your Own DKIM (BYODKIM). Update the key length that should be used for Easy DKIM. Change from using no DKIM authentication to using Easy DKIM. Change from using no DKIM authentication to using
    (
        "PutEmailIdentityDkimSigningAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to enable or disable feedback forwarding for an identity. This setting determines what happens when an identity is used to send an email that results in a bounce or complaint event. If the value is true , you receive email notifications when bounce or complaint events occur. These notifications are sent to the address that you specified in the Return-Path header of the original email. You're
    (
        "PutEmailIdentityFeedbackAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to enable or disable the custom Mail-From domain configuration for an email identity.
    (
        "PutEmailIdentityMailFromAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds an email address to the suppression list for your account or for a specific tenant. To target a tenant's suppression list, specify the TenantName parameter. If you omit TenantName , the address is added to the account-level suppression list.
    (
        "PutSuppressedDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Configure the suppression list preferences for a tenant. Use this operation to enable or disable tenant-level suppression, or to change the suppressed reasons for a tenant. When you set the suppression scope to TENANT , Amazon SES maintains a separate suppression list for the tenant. When you set the scope to ACCOUNT , the tenant uses the account-level suppression list.
    (
        "PutTenantSuppressionAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Composes an email message to multiple destinations.
    ("SendBulkEmail", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds an email address to the list of identities for your Amazon SES account in the current Amazon Web Services Region and attempts to verify it. As a result of executing this operation, a customized verification email is sent to the specified address. To use this operation, you must first create a custom verification email template. For more information about creating and using custom verification
    (
        "SendCustomVerificationEmail",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sends an email message. You can use the Amazon SES API v2 to send the following types of messages: Simple – A standard email message. When you create this type of message, you specify the sender, the recipient, and the message body, and Amazon SES assembles the message for you. Raw – A raw, MIME-formatted email message. When you send this type of email, you have to specify all of the message heade
    ("SendEmail", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Add one or more tags (keys and values) to a specified resource. A tag is a label that you optionally define and associate with a resource. Tags can help you categorize and manage resources in different ways, such as by purpose, owner, environment, or other criteria. A resource can have as many as 50 tags. Each tag consists of a required tag key and an associated tag value , both of which you defin
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a preview of the MIME content of an email when provided with a template and a set of replacement data. You can execute this operation no more than once per second.
    (
        "TestRenderEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Remove one or more tags (keys and values) from a specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing configuration set. This operation performs a partial update. Only the attributes that you include in the request are updated; any omitted attribute is left unchanged.
    (
        "UpdateConfigurationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update the configuration of an event destination for a configuration set. Events include message sends, deliveries, opens, clicks, bounces, and complaints. Event destinations are places that you can send information about these events to. For example, you can send event data to Amazon EventBridge and associate a rule to send the event to the specified target.
    (
        "UpdateConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a contact's preferences for a list. You must specify all existing topic preferences in the TopicPreferences object, not just the ones that need updating; otherwise, all your existing preferences will be removed.
    ("UpdateContact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates contact list metadata. This operation does a complete replacement.
    ("UpdateContactList", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing custom verification email template. For more information about custom verification email templates, see Using custom verification email templates in the Amazon SES Developer Guide . You can execute this operation no more than once per second.
    (
        "UpdateCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified sending authorization policy for the given identity (an email address or a domain). This API returns successfully even if a policy with the specified name does not exist. This API is for the identity owner only. If you have not verified the identity, this API will return an error. Sending authorization is a feature that enables an identity owner to authorize other senders to
    (
        "UpdateEmailIdentityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an email template. Email templates enable you to send personalized email to one or more destinations in a single API operation. For more information, see the Amazon SES Developer Guide . You can execute this operation no more than once per second.
    (
        "UpdateEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update the customer-managed sending status for a reputation entity. This allows you to enable, disable, or reinstate sending for the entity. The customer-managed status works in conjunction with the Amazon Web Services Amazon SES-managed status to determine the overall sending capability. When you update the customer-managed status, the Amazon Web Services Amazon SES-managed status remains unchang
    (
        "UpdateReputationEntityCustomerManagedStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update the reputation management policy for a reputation entity. The policy determines how the entity responds to reputation findings, such as automatically pausing sending when certain thresholds are exceeded. Reputation management policies are Amazon Web Services Amazon SES-managed (predefined policies). You can select from none, standard, and strict policies.
    (
        "UpdateReputationEntityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
];
