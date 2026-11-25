use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AcceptAdministratorInvitation",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("AcceptInvitation", Decision::Denied(DenyReason::Mutating)),
    (
        "BatchDeleteAutomationRules",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDisableStandards",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchEnableStandards",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchGetAutomationRules",
        Decision::Deferred(
            "Required selectors AutomationRulesArns are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Rules, UnprocessedAutomationRules. Pinned purpose: Retrieves a list of details for automation rules based on rule Amazon Resource Names (ARNs).",
        ),
    ),
    (
        "BatchGetConfigurationPolicyAssociations",
        Decision::Deferred(
            "Required selectors ConfigurationPolicyAssociationIdentifiers are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationPolicyAssociations, UnprocessedConfigurationPolicyAssociations. Pinned purpose: Returns associations between an Security Hub CSPM configuration and a batch of target accounts, organizational units, or the root. Only the Security Hub CSPM delegated administrator can invoke this operation from the home Region. A configuration can ",
        ),
    ),
    (
        "BatchGetSecurityControls",
        Decision::Deferred(
            "Required selectors SecurityControlIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SecurityControls, UnprocessedIds. Pinned purpose: Provides details about a batch of security controls for the current Amazon Web Services account and Amazon Web Services Region.",
        ),
    ),
    (
        "BatchGetStandardsControlAssociations",
        Decision::Deferred(
            "Required selectors StandardsControlAssociationIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StandardsControlAssociationDetails, UnprocessedAssociations. Pinned purpose: For a batch of security controls and standards, identifies whether each control is currently enabled or disabled in a standard. Calls to this operation return a RESOURCE_NOT_FOUND_EXCEPTION error when the standard subscription for the association has",
        ),
    ),
    (
        "BatchImportFindings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateAutomationRules",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateFindings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateFindingsV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateStandardsControlAssociations",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CancelExportJobV2", Decision::Denied(DenyReason::Mutating)),
    ("CreateActionTarget", Decision::Denied(DenyReason::Mutating)),
    ("CreateAggregatorV2", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAutomationRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAutomationRuleV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateConfigurationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateConnector", Decision::Denied(DenyReason::Mutating)),
    ("CreateConnectorV2", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateFindingAggregator",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateInsight", Decision::Denied(DenyReason::Mutating)),
    ("CreateMembers", Decision::Denied(DenyReason::Mutating)),
    ("CreateTicketV2", Decision::Denied(DenyReason::Mutating)),
    ("DeclineInvitations", Decision::Denied(DenyReason::Mutating)),
    ("DeleteActionTarget", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAggregatorV2", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAutomationRuleV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteConfigurationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteConnector", Decision::Denied(DenyReason::Mutating)),
    ("DeleteConnectorV2", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteFindingAggregator",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteInsight", Decision::Denied(DenyReason::Mutating)),
    ("DeleteInvitations", Decision::Denied(DenyReason::Mutating)),
    ("DeleteMembers", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeOrganizationConfiguration",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AutoEnable, MemberAccountLimitReached, AutoEnableStandards, OrganizationConfiguration. Pinned request/ownership contract: Returns information about the way your organization is configured in Security Hub CSPM. Only the Security Hub CSPM administrator account can invoke this operation.",
        ),
    ),
    (
        "DescribeProducts",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeProductsV2",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeSecurityHubV2",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members HubV2Arn, SubscribedAt, Features. Pinned request/ownership contract: Returns details about the service resource in your account.",
        ),
    ),
    (
        "DescribeStandards",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeStandardsControls",
        Decision::Deferred(
            "Required selectors StandardsSubscriptionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Controls, NextToken. Pinned purpose: Returns a list of security standards controls. For each control, the results include information about whether it is currently enabled, the severity, and a link to remediation information. This operation returns an empty list for standard subscriptio",
        ),
    ),
    (
        "DisableImportFindingsForProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisableOrganizationAdminAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DisableSecurityHub", Decision::Denied(DenyReason::Mutating)),
    (
        "DisableSecurityHubFeatureV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisableSecurityHubV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateFromAdministratorAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateFromMasterAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateMembers",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableImportFindingsForProduct",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableOrganizationAdminAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("EnableSecurityHub", Decision::Denied(DenyReason::Mutating)),
    (
        "EnableSecurityHubFeatureV2",
        Decision::Denied(DenyReason::Mutating),
    ),
];
