use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListFilters",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "filters",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListCisScanConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "scanConfigurations",
                        policy::arn("scanConfigurationArn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["scanConfigurationArn"],
            )?,
        ),
        (
            "ListCodeSecurityScanConfigurations",
            policy::query(
                policy::enumerate(
                    "configurations",
                    policy::arn("scanConfigurationArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["scanConfigurationArn"],
            )?,
        ),
        (
            "ListCodeSecurityIntegrations",
            policy::query(
                policy::enumerate(
                    "integrations",
                    policy::arn("integrationArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["integrationArn"],
            )?,
        ),
        (
            "ListConnectors",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "items",
                        policy::arn("connectorArn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &[
                    "connectorArn",
                    "awsConfigConnectorArn",
                    "scopeConfiguration",
                    "enablementStatus",
                ],
            )?,
        ),
        (
            "ListConnectorScanConfigurations",
            policy::query(
                policy::enumerate(
                    "scanConfigurations",
                    policy::arn("awsConfigConnectorArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &[
                    "awsConfigConnectorArn",
                    "connectorArns",
                    "scanConfiguration",
                ],
            )?,
        ),
    ];
    policy::finish(
        "inspector2",
        approved,
        DEFERRED,
        "Filter, scan-configuration/integration and connector metadata are approved without optional filters. Full connector scope/health and scan duration structures contain IDs, enums, status and dates, not credentials or findings. Scan configuration is keyed by its AWS Config connector reference, not a fabricated Inspector ARN. Findings, code snippets, coverage and organization traversal stay unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

const DEFERRED: &[&str] = &[
    "BatchGetAccountStatus",
    "BatchGetFindingDetails",
    "BatchGetFreeTrialInfo",
    "BatchGetMemberEc2DeepInspectionStatus",
    "DescribeOrganizationConfiguration",
    "GetCisScanReport",
    "GetCisScanResultDetails",
    "GetClustersForImage",
    "GetCodeSecurityIntegration",
    "GetCodeSecurityScan",
    "GetCodeSecurityScanConfiguration",
    "GetConfiguration",
    "GetDelegatedAdminAccount",
    "GetEc2DeepInspectionConfiguration",
    "GetEncryptionKey",
    "GetMember",
    "GetOnDemandScanStatus",
    "ListAccountPermissions",
    "ListCisScanResultsAggregatedByChecks",
    "ListCisScanResultsAggregatedByTargetResource",
    "ListCisScans",
    "ListCodeSecurityScanConfigurationAssociations",
    "ListCoverage",
    "ListCoverageStatistics",
    "ListDelegatedAdminAccounts",
    "ListFindingAggregations",
    "ListFindings",
    "ListMembers",
    "ListOnDemandScans",
    "ListTagsForResource",
    "ListUsageTotals",
    "SearchVulnerabilities",
];

const MUTATING: &[&str] = &[
    "AssociateMember",
    "BatchAssociateCodeSecurityScanConfiguration",
    "BatchDisassociateCodeSecurityScanConfiguration",
    "BatchUpdateMemberEc2DeepInspectionStatus",
    "CancelFindingsReport",
    "CancelSbomExport",
    "CreateCisScanConfiguration",
    "CreateCodeSecurityIntegration",
    "CreateCodeSecurityScanConfiguration",
    "CreateConnector",
    "CreateFilter",
    "CreateFindingsReport",
    "CreateSbomExport",
    "DeleteCisScanConfiguration",
    "DeleteCodeSecurityIntegration",
    "DeleteCodeSecurityScanConfiguration",
    "DeleteConnector",
    "DeleteFilter",
    "Disable",
    "DisableDelegatedAdminAccount",
    "DisassociateMember",
    "Enable",
    "EnableDelegatedAdminAccount",
    "ResetEncryptionKey",
    "SendCisSessionHealth",
    "SendCisSessionTelemetry",
    "StartCisSession",
    "StartCodeSecurityScan",
    "StartOnDemandScan",
    "StopCisSession",
    "TagResource",
    "UntagResource",
    "UpdateCisScanConfiguration",
    "UpdateCodeSecurityIntegration",
    "UpdateCodeSecurityScanConfiguration",
    "UpdateConfiguration",
    "UpdateConnector",
    "UpdateConnectorScanConfiguration",
    "UpdateEc2DeepInspectionConfiguration",
    "UpdateEncryptionKey",
    "UpdateFilter",
    "UpdateOrgEc2DeepInspectionConfiguration",
    "UpdateOrganizationConfiguration",
];

const CONTENT: &[&str] = &[
    "BatchGetCodeSnippet",
    "GetFindingsReportStatus",
    "GetSbomExport",
];
