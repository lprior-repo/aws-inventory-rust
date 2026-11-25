use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateEmailIdentityCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchGetMetricData",
        Decision::Deferred(
            "Required selectors Queries are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Results, Errors. Pinned purpose: Retrieves batches of metric data collected based on your sending activity. You can execute this operation no more than 16 times per second, and with at most 160 queries from the batches per second (cumulative).",
        ),
    ),
    ("CancelExportJob", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateConfigurationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateContact", Decision::Denied(DenyReason::Mutating)),
    ("CreateContactList", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateDedicatedIpPool",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateDeliverabilityTestReport",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEmailIdentity",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEmailIdentityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateExportJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateImportJob", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateMultiRegionEndpoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTenant", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTenantResourceAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteConfigurationSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteConfigurationSetEventDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteContact", Decision::Denied(DenyReason::Mutating)),
    ("DeleteContactList", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteDedicatedIpPool",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEmailIdentity",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEmailIdentityPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEmailTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteMultiRegionEndpoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteSuppressedDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTenant", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTenantResourceAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateEmailIdentityCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAccount",
        Decision::Deferred(
            "Account sending configuration has no immutable resource identifier in its output. Current identity grammar cannot use the implicit request account as a singleton key.",
        ),
    ),
    (
        "GetBlacklistReports",
        Decision::Deferred(
            "Required selectors BlacklistItemNames are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members BlacklistReport. Pinned purpose: Retrieve a list of the blacklists that your dedicated IP addresses appear on.",
        ),
    ),
    (
        "GetConfigurationSet",
        Decision::Deferred(
            "Required selectors ConfigurationSetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationSetName, TrackingOptions, DeliveryOptions, ReputationOptions, SendingOptions, Tags, SuppressionOptions, VdmOptions, ArchivingOptions, MessageSecurityOptions. Pinned purpose: Get information about an existing configuration set, including the dedicated IP pool that it's associated with, whether or not it's enabled for sending email, and more. Configuration sets are groups of rules that you can apply to the emails you send.",
        ),
    ),
    (
        "GetConfigurationSetEventDestinations",
        Decision::Deferred(
            "Required selectors ConfigurationSetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EventDestinations. Pinned purpose: Retrieve a list of event destinations that are associated with a configuration set. Events include message sends, deliveries, opens, clicks, bounces, and complaints. Event destinations are places that you can send information about these events to. F",
        ),
    ),
    ("GetContact", Decision::Denied(DenyReason::Content)),
    (
        "GetContactList",
        Decision::Deferred(
            "Required selectors ContactListName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ContactListName, Topics, Description, CreatedTimestamp, LastUpdatedTimestamp, Tags. Pinned purpose: Returns contact list metadata. It does not return any information about the contacts present in the list.",
        ),
    ),
    (
        "GetCustomVerificationEmailTemplate",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetDedicatedIp",
        Decision::Deferred(
            "Required selectors Ip are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DedicatedIp. Pinned purpose: Get information about a dedicated IP address, including the name of the dedicated IP pool that it's associated with, as well information about the automatic warm-up process for the address.",
        ),
    ),
    (
        "GetDedicatedIpPool",
        Decision::Deferred(
            "Required selectors PoolName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DedicatedIpPool. Pinned purpose: Retrieve information about the dedicated pool.",
        ),
    ),
    (
        "GetDeliverabilityDashboardOptions",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members DashboardEnabled, SubscriptionExpiryDate, AccountStatus, ActiveSubscribedDomains, PendingExpirationSubscribedDomains. Pinned request/ownership contract: Retrieve information about the status of the Deliverability dashboard for your account. When the Deliverability dashboard is enabled, you gain access to reputation, deliverability, and other metrics for the domains that you use to send email. You also gain the ability to perform predictive inbox placement tests. When you use the Deliverability dashboard, you pay a monthly subscription charge, in addition to any other fees that you accrue by using",
        ),
    ),
    (
        "GetDeliverabilityTestReport",
        Decision::Deferred(
            "Required selectors ReportId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DeliverabilityTestReport, OverallPlacement, IspPlacements, Message, Tags. Pinned purpose: Retrieve the results of a predictive inbox placement test.",
        ),
    ),
    (
        "GetDomainDeliverabilityCampaign",
        Decision::Deferred(
            "Required selectors CampaignId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainDeliverabilityCampaign. Pinned purpose: Retrieve all the deliverability data for a specific campaign. This data is available for a campaign only if the campaign sent email by using a domain that the Deliverability dashboard is enabled for.",
        ),
    ),
    (
        "GetDomainStatisticsReport",
        Decision::Deferred(
            "Required selectors Domain, StartDate, EndDate are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OverallVolume, DailyVolumes. Pinned purpose: Retrieve inbox placement and engagement rates for the domains that you use to send email.",
        ),
    ),
    (
        "GetEmailAddressInsights",
        Decision::Deferred(
            "Required selectors EmailAddress are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MailboxValidation. Pinned purpose: Provides validation insights about a specific email address, including syntax validation, DNS record checks, mailbox existence, and other deliverability factors.",
        ),
    ),
    (
        "GetEmailIdentity",
        Decision::Deferred(
            "Required selectors EmailIdentity are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members IdentityType, FeedbackForwardingStatus, VerifiedForSendingStatus, DkimAttributes, MailFromAttributes, Policies, Tags, ConfigurationSetName, VerificationStatus, VerificationInfo. Pinned purpose: Provides information about a specific identity, including the identity's verification status, sending authorization policies, its DKIM authentication status, and its custom Mail-From settings.",
        ),
    ),
    (
        "GetEmailIdentityPolicies",
        Decision::Deferred(
            "Required selectors EmailIdentity are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policies. Pinned purpose: Returns the requested sending authorization policies for the given identity (an email address or a domain). The policies are returned as a map of policy names to policy contents. You can retrieve a maximum of 20 policies at a time. This API is for th",
        ),
    ),
    ("GetEmailTemplate", Decision::Denied(DenyReason::Content)),
    (
        "GetExportJob",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobId, ExportSourceType, JobStatus, ExportDestination, ExportDataSource, CreatedTimestamp, CompletedTimestamp, FailureInfo, Statistics. Pinned purpose: Provides information about an export job. When the job status is COMPLETED , the response includes a pre-signed URL in ExportDestination.S3Url that you use to download the export file.",
        ),
    ),
    (
        "GetImportJob",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobId, ImportDestination, ImportDataSource, FailureInfo, JobStatus, CreatedTimestamp, CompletedTimestamp, ProcessedRecordsCount, FailedRecordsCount. Pinned purpose: Provides information about an import job.",
        ),
    ),
    ("GetMessageInsights", Decision::Denied(DenyReason::Content)),
    (
        "GetMultiRegionEndpoint",
        Decision::Deferred(
            "Required selectors EndpointName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EndpointName, EndpointId, Routes, Status, CreatedTimestamp, LastUpdatedTimestamp. Pinned purpose: Displays the multi-region endpoint (global-endpoint) configuration. Only multi-region endpoints (global-endpoints) whose primary region is the AWS-Region where operation is executed can be displayed.",
        ),
    ),
    (
        "GetReputationEntity",
        Decision::Deferred(
            "Required selectors ReputationEntityReference, ReputationEntityType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ReputationEntity. Pinned purpose: Retrieve information about a specific reputation entity, including its reputation management policy, customer-managed status, Amazon Web Services Amazon SES-managed status, and aggregate sending status. Reputation entities represent resources in your",
        ),
    ),
    (
        "GetSuppressedDestination",
        Decision::Denied(DenyReason::Content),
    ),
];
