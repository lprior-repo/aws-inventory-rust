// Pinned operation purposes: .local/aws-models/models/quicksight/service/2018-04-01/quicksight-2018-04-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a DLP setting configuration from an Amazon Web Services account.
    ("DeleteDlpSetting", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Permanently deletes a flow from the specified Amazon Web Services account. This operation cannot be undone.
    ("DeleteFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an empty folder.
    ("DeleteFolder", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes an asset, such as a dashboard, analysis, or dataset, from a folder.
    (
        "DeleteFolderMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a user group from Amazon Quick Sight.
    ("DeleteGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a user from a group so that the user is no longer a member of the group.
    (
        "DeleteGroupMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an existing IAM policy assignment.
    (
        "DeleteIAMPolicyAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes all access scopes and authorized targets that are associated with a service from the Quick Sight IAM Identity Center application. This operation is only supported for Quick Sight accounts that use IAM Identity Center.
    (
        "DeleteIdentityPropagationConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a knowledge base.
    (
        "DeleteKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a limits profile.
    (
        "DeleteLimitsProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a namespace and the users and groups that are associated with the namespace. This is an asynchronous process. Assets including dashboards, analyses, datasets and data sources are not deleted. To delete these assets, you use the API operations for the relevant asset.
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an OAuthClientApplication.
    (
        "DeleteOAuthClientApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a refresh schedule from a dataset.
    (
        "DeleteRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes custom permissions from the role.
    (
        "DeleteRoleCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a group from a role.
    (
        "DeleteRoleMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon QuickSight space.
    ("DeleteSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a template.
    ("DeleteTemplate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the item that the specified template alias points to. If you provide a specific alias, you delete the version of the template that the alias points to.
    (
        "DeleteTemplateAlias",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a theme.
    ("DeleteTheme", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the version of the theme that the specified theme alias points to. If you provide a specific alias, you delete the version of the theme that the alias points to.
    ("DeleteThemeAlias", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a topic.
    ("DeleteTopic", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a topic refresh schedule.
    (
        "DeleteTopicRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Q topic.
    ("DeleteTopicV2", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the Amazon Quick Sight user that is associated with the identity of the IAM user or role that's making the call. The IAM user isn't deleted as a result of this call.
    ("DeleteUser", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a user identified by its principal ID.
    (
        "DeleteUserByPrincipalId",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a custom permissions profile from a user.
    (
        "DeleteUserCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a VPC connection.
    (
        "DeleteVPCConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeAccountCustomPermission",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CustomPermissionsName, RequestId, Status. Pinned purpose: Describes the custom permissions profile that is applied to an account.",
        ),
    ),
    (
        "DescribeAccountCustomization",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, AwsAccountId, Namespace, AccountCustomization, RequestId, Status. Pinned purpose: Describes the customizations associated with the provided Amazon Web Services account and Amazon Quick Sight namespace. The Quick Sight console evaluates which customizations to apply by running this API operation with the Resolved flag included. To ",
        ),
    ),
    (
        "DescribeAccountSettings",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountSettings, RequestId, Status. Pinned purpose: Describes the settings that were used when your Quick Sight subscription was first created in this Amazon Web Services account.",
        ),
    ),
    (
        "DescribeAccountSubscription",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountInfo, Status, RequestId. Pinned purpose: Use the DescribeAccountSubscription operation to receive a description of an Quick Sight account's subscription. A successful API call returns an AccountInfo object that includes an account's name, subscription status, authentication type, edition, a",
        ),
    ),
    (
        "DescribeActionConnector",
        Decision::Deferred(
            "Required selectors AwsAccountId, ActionConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ActionConnector, RequestId, Status. Pinned purpose: Retrieves detailed information about an action connector, including its configuration, authentication settings, enabled actions, and current status.",
        ),
    ),
    (
        "DescribeActionConnectorPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, ActionConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, ActionConnectorId, Permissions, RequestId, Status. Pinned purpose: Retrieves the permissions configuration for an action connector, showing which users, groups, and namespaces have access and what operations they can perform.",
        ),
    ),
    (
        "DescribeAgent",
        Decision::Deferred(
            "Required selectors AgentId, AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Agent, RequestId. Pinned purpose: Describes an agent.",
        ),
    ),
    (
        "DescribeAgentPermissions",
        Decision::Deferred(
            "Required selectors AgentId, AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, AgentId, Permissions, RequestId. Pinned purpose: Describes the resource permissions for an agent.",
        ),
    ),
    (
        "DescribeAnalysis",
        Decision::Deferred(
            "Required selectors AwsAccountId, AnalysisId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Analysis, Status, RequestId. Pinned purpose: Provides a summary of the metadata for an analysis.",
        ),
    ),
    (
        "DescribeAnalysisDefinition",
        Decision::Deferred(
            "Required selectors AwsAccountId, AnalysisId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AnalysisId, Name, Errors, ResourceStatus, ThemeArn, Definition, Status, RequestId. Pinned purpose: Provides a detailed description of the definition of an analysis. If you do not need to know details about the content of an Analysis, for instance if you are trying to check the status of a recently created or updated Analysis, use the DescribeAnaly",
        ),
    ),
    (
        "DescribeAnalysisPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, AnalysisId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AnalysisId, AnalysisArn, Permissions, Status, RequestId. Pinned purpose: Provides the read and write permissions for an analysis.",
        ),
    ),
    (
        "DescribeApp",
        Decision::Deferred(
            "Required selectors AwsAccountId, AppId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members App, RequestId. Pinned purpose: Describes an app.",
        ),
    ),
    (
        "DescribeAppPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, AppId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AppId, Arn, Permissions, RequestId. Pinned purpose: Describes the resource permissions for an app.",
        ),
    ),
    (
        "DescribeApprovalPolicy",
        Decision::Deferred(
            "Required selectors PolicyId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: Describes an approval policy in Quick Sight.",
        ),
    ),
    (
        "DescribeAssetBundleExportJob",
        Decision::Deferred(
            "Required selectors AwsAccountId, AssetBundleExportJobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, DownloadUrl, Errors, Arn, CreatedTime, AssetBundleExportJobId, AwsAccountId, ResourceArns, IncludeAllDependencies, ExportFormat, CloudFormationOverridePropertyConfiguration, RequestId, Status, IncludePermissions, IncludeTags, ValidationStrategy, Warnings, IncludeFolderMemberships, IncludeFolderMembers. Pinned purpose: Describes an existing export job. Poll job descriptions after a job starts to know the status of the job. When a job succeeds, a URL is provided to download the exported assets' data from. Download URLs are valid for five minutes after they are gener",
        ),
    ),
    // Full modeled response contains authentication/secret material OverrideParameters.DataSources[].Credentials.CredentialPair.Password. Pinned purpose: Describes an existing import job. Poll job descriptions after starting a job to know when it has succeeded or failed. Job descriptions are available for 14 days after job starts.. Projection cannot authorize this call.
    (
        "DescribeAssetBundleImportJob",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "DescribeAutomationJob",
        Decision::Deferred(
            "Required selectors AwsAccountId, AutomationGroupId, AutomationId, JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, CreatedAt, StartedAt, EndedAt, JobStatus, InputPayload, OutputPayload, RequestId. Pinned purpose: Retrieves the status and details of a specified automation job, including its status and outputs.",
        ),
    ),
    (
        "DescribeBrand",
        Decision::Deferred(
            "Required selectors AwsAccountId, BrandId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RequestId, BrandDetail, BrandDefinition. Pinned purpose: Describes a brand.",
        ),
    ),
    (
        "DescribeBrandAssignment",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RequestId, BrandArn. Pinned purpose: Describes a brand assignment.",
        ),
    ),
    (
        "DescribeBrandPublishedVersion",
        Decision::Deferred(
            "Required selectors AwsAccountId, BrandId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RequestId, BrandDetail, BrandDefinition. Pinned purpose: Describes the published version of the brand.",
        ),
    ),
    (
        "DescribeCustomPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, CustomPermissionsName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, CustomPermissions, RequestId. Pinned purpose: Describes a custom permissions profile.",
        ),
    ),
    (
        "DescribeDashboard",
        Decision::Deferred(
            "Required selectors AwsAccountId, DashboardId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Dashboard, Status, RequestId. Pinned purpose: Provides a summary for a dashboard.",
        ),
    ),
    (
        "DescribeDashboardDefinition",
        Decision::Deferred(
            "Required selectors AwsAccountId, DashboardId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DashboardId, Errors, Name, ResourceStatus, ThemeArn, Definition, Status, RequestId, DashboardPublishOptions. Pinned purpose: Provides a detailed description of the definition of a dashboard. If you do not need to know details about the content of a dashboard, for instance if you are trying to check the status of a recently created or updated dashboard, use the DescribeDash",
        ),
    ),
    (
        "DescribeDashboardPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, DashboardId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DashboardId, DashboardArn, Permissions, Status, RequestId, LinkSharingConfiguration. Pinned purpose: Describes read and write permissions for a dashboard.",
        ),
    ),
    (
        "DescribeDashboardSnapshotJob",
        Decision::Deferred(
            "Required selectors AwsAccountId, DashboardId, SnapshotJobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AwsAccountId, DashboardId, SnapshotJobId, UserConfiguration, SnapshotConfiguration, Arn, JobStatus, CreatedTime, LastUpdatedTime, RequestId, Status. Pinned purpose: Describes an existing snapshot job. Poll job descriptions after a job starts to know the status of the job. For information on available status codes, see JobStatus . Registered user support This API can be called as before to get status of a job sta",
        ),
    ),
    (
        "DescribeDashboardSnapshotJobResult",
        Decision::Deferred(
            "Required selectors AwsAccountId, DashboardId, SnapshotJobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, JobStatus, CreatedTime, LastUpdatedTime, Result, ErrorInfo, RequestId, Status. Pinned purpose: Describes the result of an existing snapshot job that has finished running. A finished snapshot job will return a COMPLETED or FAILED status when you poll the job with a DescribeDashboardSnapshotJob API call. If the job has not finished running, this",
        ),
    ),
    (
        "DescribeDashboardsQAConfiguration",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DashboardsQAStatus, RequestId, Status. Pinned purpose: Describes an existing dashboard QA configuration.",
        ),
    ),
    (
        "DescribeDataSet",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSetId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataSet, RequestId, Status. Pinned purpose: Describes a dataset.",
        ),
    ),
];
