use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("StartAutomationJob", Decision::Denied(DenyReason::Mutating)),
    (
        "StartDashboardSnapshotJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartDashboardSnapshotJobSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAccountCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateAccountCustomization",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateAccountSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateActionConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateActionConnectorPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateAgent", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAgentPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateAnalysis", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAnalysisPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateAppPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateApplicationWithTokenExchangeGrant",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateApprovalPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateBrand", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateBrandAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateBrandPublishedVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateCustomPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateDashboard", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDashboardLinks",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDashboardPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDashboardPublishedVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDashboardsQAConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateDataSet", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDataSetPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateDataSource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDataSourcePermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDefaultQBusinessApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateDlpSetting", Decision::Denied(DenyReason::Mutating)),
    ("UpdateFlow", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFlowPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateFolder", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFolderPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateIAMPolicyAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateIdentityPropagationConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateIpRestriction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateKeyRegistration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateKnowledgeBasePermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateLimitsProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateOAuthClientApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdatePublicSharingSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateQPersonalizationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateQuickSightQSearchConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResourcePermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateRoleCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSPICECapacityConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSelfUpgrade", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateSelfUpgradeConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSpace", Decision::Denied(DenyReason::Mutating)),
];
