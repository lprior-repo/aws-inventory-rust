use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchCreateTopicReviewedAnswer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteTopicReviewedAnswer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDescribeUserLimits",
        Decision::Deferred(
            "Required selectors accountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members userLimits, errors. Pinned purpose: Describes the effective resource limits for one or more Amazon Quick Sight users, including the limits that apply to each user based on their profile assignments.",
        ),
    ),
    ("CancelIngestion", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAccountCustomization",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAccountSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateActionConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAgent", Decision::Denied(DenyReason::Mutating)),
    ("CreateAnalysis", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateApprovalPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateBrand", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateCustomPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateDashboard", Decision::Denied(DenyReason::Mutating)),
    ("CreateDataSet", Decision::Denied(DenyReason::Mutating)),
    ("CreateDataSource", Decision::Denied(DenyReason::Mutating)),
    ("CreateDlpSetting", Decision::Denied(DenyReason::Mutating)),
    ("CreateFlow", Decision::Denied(DenyReason::Mutating)),
    ("CreateFolder", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateFolderMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateGroupMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateIAMPolicyAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateIngestion", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateLimitsProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateNamespace", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateOAuthClientApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateRoleMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSpace", Decision::Denied(DenyReason::Mutating)),
    ("CreateTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTemplateAlias",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTheme", Decision::Denied(DenyReason::Mutating)),
    ("CreateThemeAlias", Decision::Denied(DenyReason::Mutating)),
    ("CreateTopic", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTopicRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTopicV2", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateVPCConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccountCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccountCustomization",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAccountSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteActionConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAgent", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAnalysis", Decision::Denied(DenyReason::Mutating)),
    ("DeleteApp", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteApprovalPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteBrand", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteBrandAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteCustomPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteDashboard", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDataSet", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteDataSetRefreshProperties",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteDataSource", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteDefaultQBusinessApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
];
