// Pinned operation purposes: .local/aws-models/models/quicksight/service/2018-04-01/quicksight-2018-04-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Starts a new job for a specified automation. The job runs the automation with the provided input payload.
    ("StartAutomationJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts an asynchronous job that generates a snapshot of a dashboard's output. You can request one or several of the following format configurations in each API call. 1 PDF 1 Excel workbook that includes up to 5 table or pivot table visuals 5 CSVs from table or pivot table visuals Exporting CSV, Excel, or Pixel Perfect PDF reports requires Pixel Perfect Report Add-on. The status of a submitted job 
    ("StartDashboardSnapshotJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts an asynchronous job that runs an existing dashboard schedule and sends the dashboard snapshot through email. Only one job can run simultaneously in a given schedule. Repeated requests are skipped with a 202 HTTP status code. For more information, see Scheduling and sending Amazon Quick Sight reports by email and Configuring email report settings for a Amazon Quick Sight dashboard in the Ama
    ("StartDashboardSnapshotJobSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns one or more tags (key-value pairs) to the specified Amazon Quick Sight resource. Tags can help you organize and categorize your resources. You can also use them to scope user permissions, by granting a user permission to access or change only resources with certain tag values. You can use the TagResource operation with a resource that already has tags. If you specify a new tag key for the 
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag or tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Applies a custom permissions profile to an account.
    ("UpdateAccountCustomPermission", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates Amazon Quick Sight customizations. Currently, the only customization that you can use is a theme. You can use customizations for your Amazon Web Services account or, if you specify a namespace, for a Quick Sight namespace instead. Customizations that apply to a namespace override customizations that apply to an Amazon Web Services account. To find out which customizations apply, use the De
    ("UpdateAccountCustomization", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the Amazon Quick Sight settings in your Amazon Web Services account.
    ("UpdateAccountSettings", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing action connector with new configuration details, authentication settings, or enabled actions. You can modify the connector's name, description, authentication configuration, and which actions are enabled. For more information, https://docs.aws.amazon.com/quicksuite/latest/userguide/quick-action-auth.html .
    ("UpdateActionConnector", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the permissions for an action connector by granting or revoking access for specific users and groups. You can control who can view, use, or manage the action connector.
    ("UpdateActionConnectorPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing agent.
    ("UpdateAgent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource permissions for an agent.
    ("UpdateAgentPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an analysis in Amazon Quick Sight
    ("UpdateAnalysis", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the read and write permissions for an analysis.
    ("UpdateAnalysisPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource permissions for an app. You can grant or revoke permissions and, optionally, change the app's visibility.
    ("UpdateAppPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Quick application with a token exchange grant. This operation only supports Quick applications that are registered with IAM Identity Center.
    ("UpdateApplicationWithTokenExchangeGrant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an approval policy in Quick Sight.
    ("UpdateApprovalPolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a brand.
    ("UpdateBrand", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a brand assignment.
    ("UpdateBrandAssignment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the published version of a brand.
    ("UpdateBrandPublishedVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a custom permissions profile.
    ("UpdateCustomPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a dashboard in an Amazon Web Services account. Updating a Dashboard creates a new dashboard version but does not immediately publish the new version. You can update the published version of a dashboard by using the UpdateDashboardPublishedVersion API operation.
    ("UpdateDashboard", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the linked analyses on a dashboard.
    ("UpdateDashboardLinks", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates read and write permissions on a dashboard.
    ("UpdateDashboardPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the published version of a dashboard.
    ("UpdateDashboardPublishedVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a Dashboard QA configuration.
    ("UpdateDashboardsQAConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a dataset. Partial updates are not supported by this operation.
    ("UpdateDataSet", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the permissions on a dataset. The permissions resource is arn:aws:quicksight:region:aws-account-id:dataset/data-set-id .
    ("UpdateDataSetPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a data source.
    ("UpdateDataSource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the permissions to a data source.
    ("UpdateDataSourcePermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a Amazon Q Business application that is linked to a Quick Sight account.
    ("UpdateDefaultQBusinessApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing DLP setting configuration in an Amazon Web Services account. Fields that are omitted from the request retain their current values.
    ("UpdateDlpSetting", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing flow. Supply only the fields you want to change. Updates both DRAFT and PUBLISHED versions. When FlowDefinition is provided, all existing steps are replaced with the new definition.
    ("UpdateFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates permissions against principals on a flow.
    ("UpdateFlowPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the name of a folder.
    ("UpdateFolder", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates permissions of a folder.
    ("UpdateFolderPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Changes a group description.
    ("UpdateGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing IAM policy assignment. This operation updates only the optional parameter or parameters that are specified in the request. This overwrites all of the users included in Identities .
    ("UpdateIAMPolicyAssignment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or updates services and authorized targets to configure what the Quick Sight IAM Identity Center application can access. This operation is only supported for Quick Sight accounts using IAM Identity Center
    ("UpdateIdentityPropagationConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the content and status of IP rules. Traffic from a source is allowed when the source satisfies either the IpRestrictionRule , VpcIdRestrictionRule , or VpcEndpointIdRestrictionRule . To use this operation, you must provide the entire map of rules. You can use the DescribeIpRestriction operation to get the current rule map.
    ("UpdateIpRestriction", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a customer managed key in a Quick Sight account.
    ("UpdateKeyRegistration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the properties of an existing knowledge base.
    ("UpdateKnowledgeBase", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource permissions for a knowledge base.
    ("UpdateKnowledgeBasePermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the properties of an existing limits profile.
    ("UpdateLimitsProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an OAuthClientApplication.
    ("UpdateOAuthClientApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This API controls public sharing settings for your entire Quick Sight account, affecting data security and access. When you enable public sharing: Dashboards can be shared publicly This setting affects your entire Amazon Web Services account and all Quick Sight users Before proceeding: Ensure you understand the security implications and have proper IAM permissions configured. Use the UpdatePublicS
    ("UpdatePublicSharingSettings", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a personalization configuration.
    ("UpdateQPersonalizationConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the state of a Quick Sight Q Search configuration.
    ("UpdateQuickSightQSearchConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a refresh schedule for a dataset.
    ("UpdateRefreshSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the service role configuration that Amazon Quick Sight uses to access resources in your Amazon Web Services account. To use this operation, you must have the quicksight:UpdateResourcePermissions and iam:PassRole permissions. Call this operation in the Amazon Web Services Region where your Amazon Quick Sight account is registered (the home Region). You can switch from a role managed by Amaz
    ("UpdateResourcePermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the custom permissions that are associated with a role.
    ("UpdateRoleCustomPermission", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the SPICE capacity configuration for a Quick Sight account.
    ("UpdateSPICECapacityConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a self-upgrade request for a Quick user by approving, denying, or verifying the request.
    ("UpdateSelfUpgrade", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the self-upgrade configuration for a Quick account.
    ("UpdateSelfUpgradeConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the metadata of an Amazon QuickSight space.
    ("UpdateSpace", Decision::Denied(DenyReason::Mutating)),
];
