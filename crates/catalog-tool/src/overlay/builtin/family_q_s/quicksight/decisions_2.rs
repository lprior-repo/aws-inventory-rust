// Pinned operation purposes: .local/aws-models/models/quicksight/service/2018-04-01/quicksight-2018-04-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DescribeDataSetPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSetId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataSetArn, DataSetId, Permissions, RequestId, Status. Pinned purpose: Describes the permissions on a dataset. The permissions resource is arn:aws:quicksight:region:aws-account-id:dataset/data-set-id .",
        ),
    ),
    (
        "DescribeDataSetRefreshProperties",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSetId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RequestId, Status, DataSetRefreshProperties. Pinned purpose: Describes the refresh properties of a dataset.",
        ),
    ),
    (
        "DescribeDataSource",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataSource, RequestId, Status. Pinned purpose: Describes a data source.",
        ),
    ),
    (
        "DescribeDataSourcePermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataSourceArn, DataSourceId, Permissions, RequestId, Status. Pinned purpose: Describes the resource permissions for a data source.",
        ),
    ),
    (
        "DescribeDefaultQBusinessApplication",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RequestId, Status, ApplicationId. Pinned purpose: Describes a Amazon Q Business application that is linked to an Quick Sight account.",
        ),
    ),
    (
        "DescribeDlpSetting",
        Decision::Deferred(
            "Required selectors AwsAccountId, DlpSettingId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DlpSetting, RequestId. Pinned purpose: Describes the full configuration of a DLP setting in an Amazon Web Services account.",
        ),
    ),
    (
        "DescribeFlow",
        Decision::Deferred(
            "Required selectors AwsAccountId, FlowId, PublishState are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Flow, RequestId, Status. Pinned purpose: Returns the full details of a flow for the latest version of the requested publish state.",
        ),
    ),
    (
        "DescribeFolder",
        Decision::Deferred(
            "Required selectors AwsAccountId, FolderId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, Folder, RequestId. Pinned purpose: Describes a folder.",
        ),
    ),
    (
        "DescribeFolderPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, FolderId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, FolderId, Arn, Permissions, RequestId, NextToken. Pinned purpose: Describes permissions for a folder.",
        ),
    ),
    (
        "DescribeFolderResolvedPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, FolderId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, FolderId, Arn, Permissions, RequestId, NextToken. Pinned purpose: Describes the folder resolved permissions. Permissions consists of both folder direct permissions and the inherited permissions from the ancestor folders.",
        ),
    ),
    (
        "DescribeGroup",
        Decision::Deferred(
            "Required selectors GroupName, AwsAccountId, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Group, RequestId, Status. Pinned purpose: Returns an Amazon Quick Sight group's description and Amazon Resource Name (ARN).",
        ),
    ),
    (
        "DescribeGroupMembership",
        Decision::Deferred(
            "Required selectors MemberName, GroupName, AwsAccountId, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members GroupMember, RequestId, Status. Pinned purpose: Use the DescribeGroupMembership operation to determine if a user is a member of the specified group. If the user exists and is a member of the specified group, an associated GroupMember object is returned.",
        ),
    ),
    (
        "DescribeIAMPolicyAssignment",
        Decision::Deferred(
            "Required selectors AwsAccountId, AssignmentName, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members IAMPolicyAssignment, RequestId, Status. Pinned purpose: Describes an existing IAM policy assignment, as specified by the assignment name.",
        ),
    ),
    (
        "DescribeIngestion",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSetId, IngestionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Ingestion, RequestId, Status. Pinned purpose: Describes a SPICE ingestion.",
        ),
    ),
    (
        "DescribeIpRestriction",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AwsAccountId, IpRestrictionRuleMap, VpcIdRestrictionRuleMap, VpcEndpointIdRestrictionRuleMap, Enabled, RequestId, Status. Pinned purpose: Provides a summary and status of IP rules.",
        ),
    ),
    (
        "DescribeKeyRegistration",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AwsAccountId, KeyRegistration, QDataKey, RequestId, Status. Pinned purpose: Describes all customer managed key registrations in a Quick Sight account.",
        ),
    ),
    (
        "DescribeKnowledgeBase",
        Decision::Deferred(
            "Required selectors AwsAccountId, KnowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members KnowledgeBase, RequestId, Status. Pinned purpose: Describes a knowledge base.",
        ),
    ),
    (
        "DescribeKnowledgeBasePermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, KnowledgeBaseId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members KnowledgeBaseArn, KnowledgeBaseId, Permissions, RequestId, Status. Pinned purpose: Describes the resource permissions for a knowledge base.",
        ),
    ),
    (
        "DescribeLimitsProfile",
        Decision::Deferred(
            "Required selectors profileId, accountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members profile. Pinned purpose: Describes the properties of an existing limits profile.",
        ),
    ),
    (
        "DescribeNamespace",
        Decision::Deferred(
            "Required selectors AwsAccountId, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Namespace, RequestId, Status. Pinned purpose: Describes the current namespace.",
        ),
    ),
    (
        "DescribeOAuthClientApplication",
        Decision::Deferred(
            "Required selectors AwsAccountId, OAuthClientApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OAuthClientApplication, RequestId, Status. Pinned purpose: Describes an OAuthClientApplication.",
        ),
    ),
    (
        "DescribeQPersonalizationConfiguration",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PersonalizationMode, RequestId, Status. Pinned purpose: Describes a personalization configuration.",
        ),
    ),
    (
        "DescribeQuickSightQSearchConfiguration",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members QSearchStatus, RequestId, Status. Pinned purpose: Describes the state of a Quick Sight Q Search configuration.",
        ),
    ),
    (
        "DescribeRefreshSchedule",
        Decision::Deferred(
            "Required selectors AwsAccountId, DataSetId, ScheduleId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RefreshSchedule, Status, RequestId, Arn. Pinned purpose: Provides a summary of a refresh schedule.",
        ),
    ),
    (
        "DescribeResourcePermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RoleType, CustomerManagedRoleArn, RequestId, Status. Pinned purpose: Describes the service role configuration for your Amazon Quick Sight account. Call this operation in the Amazon Web Services Region where your Amazon Quick Sight account is registered (the home Region).",
        ),
    ),
    (
        "DescribeRoleCustomPermission",
        Decision::Deferred(
            "Required selectors Role, AwsAccountId, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CustomPermissionsName, RequestId, Status. Pinned purpose: Describes all custom permissions that are mapped to a role.",
        ),
    ),
    (
        "DescribeSelfUpgradeConfiguration",
        Decision::Deferred(
            "Required selectors AwsAccountId, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SelfUpgradeConfiguration, RequestId, Status. Pinned purpose: Describes the self-upgrade configuration for a Quick account.",
        ),
    ),
    (
        "DescribeSpace",
        Decision::Deferred(
            "Required selectors AwsAccountId, SpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spaceId, spaceArn, Space, Contributors, RequestId. Pinned purpose: Describes an Amazon QuickSight space.",
        ),
    ),
    (
        "DescribeSpacePermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, SpaceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members spaceId, spaceArn, Permissions, RequestId. Pinned purpose: Describes the permissions for an Amazon QuickSight space.",
        ),
    ),
    (
        "DescribeTemplate",
        Decision::Deferred(
            "Required selectors AwsAccountId, TemplateId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Template, Status, RequestId. Pinned purpose: Describes a template's metadata.",
        ),
    ),
    (
        "DescribeTemplateAlias",
        Decision::Deferred(
            "Required selectors AwsAccountId, TemplateId, AliasName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TemplateAlias, Status, RequestId. Pinned purpose: Describes the template alias for a template.",
        ),
    ),
    (
        "DescribeTemplateDefinition",
        Decision::Deferred(
            "Required selectors AwsAccountId, TemplateId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Name, TemplateId, Errors, ResourceStatus, ThemeArn, Definition, Status, RequestId. Pinned purpose: Provides a detailed description of the definition of a template. If you do not need to know details about the content of a template, for instance if you are trying to check the status of a recently created or updated template, use the DescribeTemplat",
        ),
    ),
    (
        "DescribeTemplatePermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, TemplateId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TemplateId, TemplateArn, Permissions, RequestId, Status. Pinned purpose: Describes read and write permissions on a template.",
        ),
    ),
    (
        "DescribeTheme",
        Decision::Deferred(
            "Required selectors AwsAccountId, ThemeId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Theme, Status, RequestId. Pinned purpose: Describes a theme.",
        ),
    ),
    (
        "DescribeThemeAlias",
        Decision::Deferred(
            "Required selectors AwsAccountId, ThemeId, AliasName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ThemeAlias, Status, RequestId. Pinned purpose: Describes the alias for a theme.",
        ),
    ),
    (
        "DescribeThemePermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, ThemeId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ThemeId, ThemeArn, Permissions, RequestId, Status. Pinned purpose: Describes the read and write permissions for a theme.",
        ),
    ),
    (
        "DescribeTopic",
        Decision::Deferred(
            "Required selectors AwsAccountId, TopicId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, TopicId, Topic, RequestId, Status, CustomInstructions. Pinned purpose: Describes a topic.",
        ),
    ),
    (
        "DescribeTopicPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, TopicId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TopicId, TopicArn, Permissions, Status, RequestId. Pinned purpose: Describes the permissions of a topic.",
        ),
    ),
    (
        "DescribeTopicPermissionsV2",
        Decision::Deferred(
            "Required selectors AwsAccountId, TopicId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TopicId, TopicArn, Permissions, Status, RequestId. Pinned purpose: Describes the permissions of a topic.",
        ),
    ),
    (
        "DescribeTopicRefresh",
        Decision::Deferred(
            "Required selectors AwsAccountId, TopicId, RefreshId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RefreshDetails, RequestId, Status. Pinned purpose: Describes the status of a topic refresh.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a topic refresh schedule.
    (
        "DescribeTopicRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeTopicV2",
        Decision::Deferred(
            "Required selectors AwsAccountId, TopicId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, TopicId, Topic, CustomInstructions, Status, RequestId. Pinned purpose: Describes a Q topic.",
        ),
    ),
    (
        "DescribeUser",
        Decision::Deferred(
            "Required selectors UserName, AwsAccountId, Namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members User, RequestId, Status. Pinned purpose: Returns information about a user, given the user name.",
        ),
    ),
    (
        "DescribeVPCConnection",
        Decision::Deferred(
            "Required selectors AwsAccountId, VPCConnectionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members VPCConnection, RequestId, Status. Pinned purpose: Describes a VPC connection.",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Generates an embed URL that you can use to embed an Amazon Quick dashboard or visual in your website, without having to register any reader users. Before you use this action, make sure that you have configured the dashboards and permissions. The following rules apply to the generated URL: It contains a temporary bearer token. It is valid for 5 minutes after it is generated. Once redeemed within th
    (
        "GenerateEmbedUrlForAnonymousUser",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Generates an embed URL that you can use to embed an Amazon Quick experience in your website. This action can be used for any type of user registered in an Amazon Quick account. Before you use this action, make sure that you have configured the relevant Amazon Quick resource and permissions. The following rules apply to the generated URL: It contains a temporary bearer token. It is valid for 5 minu
    (
        "GenerateEmbedUrlForRegisteredUser",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Generates an embed URL that you can use to embed an Amazon Quick Sight experience in your website. This action can be used for any type of user that is registered in an Amazon Quick Sight account that uses IAM Identity Center for authentication. This API requires identity-enhanced IAM Role sessions for the authenticated user that the API call is being made for. This API uses trusted identity propa
    (
        "GenerateEmbedUrlForRegisteredUserWithIdentity",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Generates a temporary session URL and authorization code(bearer token) that you can use to embed an Amazon Quick Sight read-only dashboard in your website or application. Before you use this command, make sure that you have configured the dashboards and permissions. Currently, you can use GetDashboardEmbedURL only from the server, not from the user's browser. The following rules apply to the gener
    (
        "GetDashboardEmbedUrl",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetFlowMetadata",
        Decision::Deferred(
            "Required selectors AwsAccountId, FlowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, FlowId, Name, Description, PublishState, UserCount, RunCount, CreatedTime, LastUpdatedTime, RequestId, Status. Pinned purpose: Retrieves the metadata of a flow, not including its definition specifying the steps.",
        ),
    ),
    (
        "GetFlowPermissions",
        Decision::Deferred(
            "Required selectors AwsAccountId, FlowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, FlowId, Permissions, RequestId, Status. Pinned purpose: Get permissions for a flow.",
        ),
    ),
    (
        "GetIdentityContext",
        Decision::Deferred(
            "Required selectors AwsAccountId, UserIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, RequestId, Context. Pinned purpose: Retrieves the identity context for a Quick Sight user in a specified namespace, allowing you to obtain identity tokens that can be used with identity-enhanced IAM role sessions to call identity-aware APIs. Currently, you can call the following APIs w",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Generates a session URL and authorization code that you can use to embed the Amazon Amazon Quick Sight console in your web server code. Use GetSessionEmbedUrl where you want to provide an authoring portal that allows users to create data sources, datasets, analyses, and dashboards. The users who access an embedded Amazon Quick Sight console need belong to the author or admin security cohort. If yo
    ("GetSessionEmbedUrl", Decision::Denied(DenyReason::Content)),
];
