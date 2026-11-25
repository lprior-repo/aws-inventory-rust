use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "EnableSecurityHubV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GenerateRecommendedPolicyV2",
        Decision::Deferred(
            "Required selectors MetadataUid are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Begins the recommended policy generation to remediate a Security Hub finding. GenerateRecommendedPolicyV2 only supports findings for unused permissions.",
        ),
    ),
    (
        "GetAdministratorAccount",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Administrator. Pinned request/ownership contract: Provides the details for the Security Hub CSPM administrator account for the current member account. Can be used by both member accounts that are managed using Organizations and accounts that were invited manually.",
        ),
    ),
    (
        "GetAggregatorV2",
        Decision::Deferred(
            "Required selectors AggregatorV2Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AggregatorV2Arn, AggregationRegion, RegionLinkingMode, LinkedRegions. Pinned purpose: Returns the configuration of the specified Aggregator V2.",
        ),
    ),
    (
        "GetAutomationRuleV2",
        Decision::Deferred(
            "Required selectors Identifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RuleArn, RuleId, RuleOrder, RuleName, RuleStatus, Description, Criteria, Actions, CreatedAt, UpdatedAt. Pinned purpose: Returns an automation rule for the V2 service.",
        ),
    ),
    (
        "GetConfigurationPolicy",
        Decision::Deferred(
            "Required selectors Identifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Id, Name, Description, UpdatedAt, CreatedAt, ConfigurationPolicy. Pinned purpose: Provides information about a configuration policy. Only the Security Hub CSPM delegated administrator can invoke this operation from the home Region.",
        ),
    ),
    (
        "GetConfigurationPolicyAssociation",
        Decision::Deferred(
            "Required selectors Target are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationPolicyId, TargetId, TargetType, AssociationType, UpdatedAt, AssociationStatus, AssociationStatusMessage. Pinned purpose: Returns the association between a configuration and a target account, organizational unit, or the root. The configuration can be a configuration policy or self-managed behavior. Only the Security Hub CSPM delegated administrator can invoke this opera",
        ),
    ),
    (
        "GetConnector",
        Decision::Deferred(
            "Required selectors ConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConnectorArn, ConnectorId, Name, Description, CreatedAt, LastUpdatedAt, Health, ProviderDetail, CreatedBy, EnablementStatus. Pinned purpose: Retrieves details for a CSPM connector based on the connector ID.",
        ),
    ),
    (
        "GetConnectorV2",
        Decision::Deferred(
            "Required selectors ConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConnectorArn, ConnectorId, Name, Description, KmsKeyArn, CreatedAt, LastUpdatedAt, Health, ProviderDetail, EnablementStatus, EnablementStatusReason. Pinned purpose: Grants permission to retrieve details for a connectorV2 based on connector id.",
        ),
    ),
    (
        "GetExportJobV2",
        Decision::Deferred(
            "Required selectors ExportJobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ExportJobId, Name, Status, DataType, OutputConfiguration, Scopes, Destination, FailureCode, FailureMessage, StartedAt, EndedAt. Pinned purpose: Returns the details of a single findings export job, including its current Status , the Destination it writes to, the OutputConfiguration it was started with, and its StartedAt and EndedAt timestamps. Use this operation to poll an export job that you",
        ),
    ),
    (
        "GetFindingAggregator",
        Decision::Deferred(
            "Required selectors FindingAggregatorArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FindingAggregatorArn, FindingAggregationRegion, RegionLinkingMode, Regions. Pinned purpose: The aggregation Region is now called the home Region . Returns the current configuration in the calling account for cross-Region aggregation. A finding aggregator is a resource that establishes the home Region and any linked Regions.",
        ),
    ),
    ("GetFindingHistory", Decision::Denied(DenyReason::Content)),
    (
        "GetFindingStatisticsV2",
        Decision::Denied(DenyReason::Content),
    ),
    ("GetFindings", Decision::Denied(DenyReason::Secret)),
    (
        "GetFindingsTrendsV2",
        Decision::Deferred(
            "Required selectors StartTime, EndTime are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Granularity, TrendsMetrics, NextToken. Pinned purpose: Returns findings trend data based on the specified criteria. This operation helps you analyze patterns and changes in findings over time.",
        ),
    ),
    ("GetFindingsV2", Decision::Denied(DenyReason::Content)),
    (
        "GetInsightResults",
        Decision::Deferred(
            "Required selectors InsightArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InsightResults. Pinned purpose: Lists the results of the Security Hub CSPM insight specified by the insight ARN.",
        ),
    ),
    (
        "GetInvitationsCount",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members InvitationsCount. Pinned request/ownership contract: We recommend using Organizations instead of Security Hub CSPM invitations to manage your member accounts. For information, see Managing Security Hub CSPM administrator and member accounts with Organizations in the Security Hub CSPM User Guide . Returns the count of all Security Hub CSPM membership invitations that were sent to the calling member account, not including the currently accepted invitation.",
        ),
    ),
    (
        "GetMasterAccount",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Master. Pinned request/ownership contract: This method is deprecated. Instead, use GetAdministratorAccount . The Security Hub CSPM console continues to use GetMasterAccount . It will eventually change to use GetAdministratorAccount . Any IAM policies that specifically control access to this function must continue to use GetMasterAccount . You should also add GetAdministratorAccount to your policies to ensure that the correct permissions are in place after the console begins to use GetAdmi",
        ),
    ),
    (
        "GetMembers",
        Decision::Deferred(
            "Required selectors AccountIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Members, UnprocessedAccounts. Pinned purpose: Returns the details for the Security Hub CSPM member accounts for the specified account IDs. An administrator account can be either the delegated Security Hub CSPM administrator account for an organization or an administrator account that enabled Sec",
        ),
    ),
    (
        "GetRecommendedPolicyV2",
        Decision::Deferred(
            "Required selectors MetadataUid are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, RecommendationType, RecommendationSteps, Error, Status, ResourceArn. Pinned purpose: Retrieves the recommended policy to remediate a Security Hub finding. GetRecommendedPolicyV2 only supports findings for unused permissions.",
        ),
    ),
    ("GetRemediationsV2", Decision::Denied(DenyReason::Content)),
    (
        "GetResourcesStatisticsV2",
        Decision::Deferred(
            "Required selectors GroupByRules are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GroupByResults. Pinned purpose: Retrieves statistical information about Amazon Web Services resources and their associated security findings. You can use the Scopes parameter to define the data boundary for the query. Currently, Scopes supports AwsOrganizations , which lets you agg",
        ),
    ),
    (
        "GetResourcesTrendsV2",
        Decision::Deferred(
            "Required selectors StartTime, EndTime are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Granularity, TrendsMetrics, NextToken. Pinned purpose: Returns resource trend data based on the specified criteria. This operation helps you analyze patterns and changes in resource compliance over time.",
        ),
    ),
    ("GetResourcesV2", Decision::Denied(DenyReason::Content)),
    (
        "GetSecurityControlDefinition",
        Decision::Deferred(
            "Required selectors SecurityControlId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SecurityControlDefinition. Pinned purpose: Retrieves the definition of a security control. The definition includes the control title, description, Region availability, parameter definitions, and other details.",
        ),
    ),
    (
        "GetSoftwareResourcesV2",
        Decision::Deferred(
            "Required selectors PackageName, PackageType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SoftwareResources, NextToken. Pinned purpose: Returns a list of resources that contain a specific software package. You identify the package with PackageName and PackageType . To narrow the results to one version of the package, include PackageVersion . You can use the Scopes parameter to define",
        ),
    ),
    (
        "GetSoftwareStatisticsV2",
        Decision::Deferred(
            "Required selectors GroupByRules are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GroupByResults. Pinned purpose: Retrieves aggregated counts of software packages installed on your resources, grouped by a field that you specify. For example, you can return the most common package types or the accounts with the most installed packages. Each count is the number of",
        ),
    ),
    ("GetSoftwareV2", Decision::Denied(DenyReason::Content)),
    (
        "InviteMembers",
        Decision::Deferred(
            "Required selectors AccountIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members UnprocessedAccounts. Pinned purpose: We recommend using Organizations instead of Security Hub CSPM invitations to manage your member accounts. For information, see Managing Security Hub CSPM administrator and member accounts with Organizations in the Security Hub CSPM User Guide . Invit",
        ),
    ),
    (
        "ListConfigurationPolicyAssociations",
        Decision::Deferred(
            "Policy associations contain organization account/OU/root targets rather than one caller-owned regional resource. Requires home-Region routing plus target-kind-aware association identity.",
        ),
    ),
    (
        "ListExposuresByRemediationV2",
        Decision::Deferred(
            "Required selectors TargetUid are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Items, TargetUid, Resource, TotalCount, Trait, NextToken. Pinned purpose: Retrieves the exposure findings tied to a specific remediation target. Results are sorted by previous severity, highest first, and are paginated.",
        ),
    ),
    (
        "ListSecurityControlDefinitions",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListStandardsControlAssociations",
        Decision::Deferred(
            "Required selectors SecurityControlId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StandardsControlAssociationSummaries, NextToken. Pinned purpose: Specifies whether a control is currently enabled or disabled in each enabled standard in the calling account. This operation omits standards control associations for standard subscriptions where StandardsControlsUpdatable has value NOT_READY_FOR_UPDA",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Returns a list of tags associated with a resource.",
        ),
    ),
    (
        "RegisterConnectorV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartConfigurationPolicyAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartConfigurationPolicyDisassociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartExportJobV2", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateActionTarget", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAggregatorV2", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAutomationRuleV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateConfigurationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateConnector", Decision::Denied(DenyReason::Mutating)),
    ("UpdateConnectorV2", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFindingAggregator",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateFindings", Decision::Denied(DenyReason::Mutating)),
    ("UpdateInsight", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateOrganizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSecurityControl",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSecurityHubConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateStandardsControl",
        Decision::Denied(DenyReason::Mutating),
    ),
];
