use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "GetTenant",
        Decision::Deferred(
            "Required selectors TenantName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Tenant. Pinned purpose: Get information about a specific tenant, including the tenant's name, ID, ARN, creation timestamp, tags, sending status, and suppression attributes.",
        ),
    ),
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
    (
        "PutAccountDedicatedIpWarmupAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutAccountDetails", Decision::Denied(DenyReason::Mutating)),
    (
        "PutAccountPricingAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccountSendingAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccountSuppressionAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutAccountVdmAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetArchivingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetDeliveryOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetReputationOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetSendingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetSuppressionOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetTrackingOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutConfigurationSetVdmOptions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutDedicatedIpInPool",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutDedicatedIpPoolScalingAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutDedicatedIpWarmupAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutDeliverabilityDashboardOption",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutEmailIdentityConfigurationSetAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutEmailIdentityDkimAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutEmailIdentityDkimSigningAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutEmailIdentityFeedbackAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutEmailIdentityMailFromAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutSuppressedDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTenantSuppressionAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("SendBulkEmail", Decision::Denied(DenyReason::Mutating)),
    (
        "SendCustomVerificationEmail",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("SendEmail", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "TestRenderEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateConfigurationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateContact", Decision::Denied(DenyReason::Mutating)),
    ("UpdateContactList", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateEmailIdentityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateReputationEntityCustomerManagedStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateReputationEntityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
];
