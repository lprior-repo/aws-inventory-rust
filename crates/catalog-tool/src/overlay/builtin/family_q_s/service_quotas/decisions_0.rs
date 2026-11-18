// Pinned operation purposes: .local/aws-models/models/service-quotas/service/2019-06-24/service-quotas-2019-06-24.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Associates your quota request template with your organization. When a new Amazon Web Services account is created in your organization, the quota increase requests in the template are automatically applied to the account. You can add a quota increase request for any adjustable quota to your template.
    (
        "AssociateServiceQuotaTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Support case for an existing quota increase request. This call only creates a Support case if the request has a Pending status.
    ("CreateSupportCase", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the quota increase request for the specified quota from your quota request template.
    (
        "DeleteServiceQuotaIncreaseRequestFromTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disables your quota request template. After a template is disabled, the quota increase requests in the template are not applied to new Amazon Web Services accounts in your organization. Disabling a quota request template does not apply its quota increase requests.
    (
        "DisassociateServiceQuotaTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAWSDefaultServiceQuota",
        Decision::Deferred(
            "Required selectors ServiceCode, QuotaCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Quota. Pinned purpose: Retrieves the default value for the specified quota. The default value does not reflect any quota increases.",
        ),
    ),
    (
        "GetAssociationForServiceQuotaTemplate",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ServiceQuotaTemplateAssociationStatus. Pinned request/ownership contract: Retrieves the status of the association for the quota request template.",
        ),
    ),
    (
        "GetAutoManagementConfiguration",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members OptInLevel, OptInType, NotificationArn, OptInStatus, ExclusionList. Pinned request/ownership contract: Retrieves information about your Service Quotas Automatic Management configuration. Automatic Management monitors your Service Quotas utilization and notifies you before you run out of your allocated quotas.",
        ),
    ),
    (
        "GetQuotaUtilizationReport",
        Decision::Deferred(
            "Required selectors ReportId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ReportId, Status, GeneratedAt, TotalCount, Quotas, NextToken, ErrorCode, ErrorMessage. Pinned purpose: Retrieves the quota utilization report for your Amazon Web Services account. This operation returns paginated results showing your quota usage across all Amazon Web Services services, sorted by utilization percentage in descending order (highest util",
        ),
    ),
    (
        "GetRequestedServiceQuotaChange",
        Decision::Deferred(
            "Required selectors RequestId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RequestedQuota. Pinned purpose: Retrieves information about the specified quota increase request.",
        ),
    ),
    (
        "GetServiceQuota",
        Decision::Deferred(
            "Required selectors ServiceCode, QuotaCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Quota. Pinned purpose: Retrieves the applied quota value for the specified account-level or resource-level quota. For some quotas, only the default values are available. If the applied quota value is not available for a quota, the quota is not retrieved.",
        ),
    ),
    (
        "GetServiceQuotaIncreaseRequestFromTemplate",
        Decision::Deferred(
            "Required selectors ServiceCode, QuotaCode, AwsRegion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceQuotaIncreaseRequestInTemplate. Pinned purpose: Retrieves information about the specified quota increase request in your quota request template.",
        ),
    ),
    (
        "ListAWSDefaultServiceQuotas",
        Decision::Deferred(
            "Required selectors ServiceCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Quotas. Pinned purpose: Lists the default values for the quotas for the specified Amazon Web Services service. A default value does not reflect any quota increases.",
        ),
    ),
    (
        "ListRequestedServiceQuotaChangeHistoryByQuota",
        Decision::Deferred(
            "Required selectors ServiceCode, QuotaCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, RequestedQuotas. Pinned purpose: Retrieves the quota increase requests for the specified quota. Filter responses to return quota requests at either the account level, resource level, or all levels.",
        ),
    ),
    (
        "ListServiceQuotaIncreaseRequestsInTemplate",
        Decision::Deferred(
            "Template rows include AwsRegion and global quota flag. A single local request scope cannot represent all template target Regions without per-item scope dispatch.",
        ),
    ),
    (
        "ListServiceQuotas",
        Decision::Deferred(
            "Required selectors ServiceCode are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Quotas. Pinned purpose: Lists the applied quota values for the specified Amazon Web Services service. For some quotas, only the default values are available. If the applied quota value is not available for a quota, the quota is not retrieved. Filter responses to return appl",
        ),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists the names and codes for the Amazon Web Services services integrated with Service Quotas.
    ("ListServices", Decision::Denied(DenyReason::ReferenceData)),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceARN. Pinned purpose: Returns a list of the tags assigned to the specified applied quota.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a quota increase request to your quota request template.
    (
        "PutServiceQuotaIncreaseRequestIntoTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Submits a quota increase request for the specified quota at the account or resource level.
    (
        "RequestServiceQuotaIncrease",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts Service Quotas Automatic Management for an Amazon Web Services account, including notification preferences and excluded quotas configurations. Automatic Management monitors your Service Quotas utilization and notifies you before you run out of your allocated quotas.
    (
        "StartAutoManagement",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates the generation of a quota utilization report for your Amazon Web Services account. This asynchronous operation analyzes your quota usage across all Amazon Web Services services and returns a unique report identifier that you can use to retrieve the results. The report generation process may take several seconds to complete, depending on the number of quotas in your account. Use the GetQu
    (
        "StartQuotaUtilizationReport",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops Service Quotas Automatic Management for an Amazon Web Services account and removes all associated configurations. Automatic Management monitors your Service Quotas utilization and notifies you before you run out of your allocated quotas.
    ("StopAutoManagement", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds tags to the specified applied quota. You can include one or more tags to add to the quota.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from the specified applied quota. You can specify one or more tags to remove.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates your Service Quotas Automatic Management configuration, including notification preferences and excluded quotas. Automatic Management monitors your Service Quotas utilization and notifies you before you run out of your allocated quotas.
    (
        "UpdateAutoManagement",
        Decision::Denied(DenyReason::Mutating),
    ),
];
