// Pinned operation purposes: .local/aws-models/models/quicksight/service/2018-04-01/quicksight-2018-04-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates new reviewed answers for a Q Topic.
    (
        "BatchCreateTopicReviewedAnswer",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes one or more knowledge bases.
    (
        "BatchDeleteKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes reviewed answers for Q Topic.
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
    // Changes service state rather than reading inventory. Pinned purpose: Cancels an ongoing ingestion of data into SPICE.
    ("CancelIngestion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates Amazon Quick Sight customizations. Currently, you can add a custom default theme by using the CreateAccountCustomization or UpdateAccountCustomization API operation. To further customize Amazon Quick Sight by removing Amazon Quick Sight sample assets and videos for all new users, see Customizing Quick Sight in the Amazon Quick Sight User Guide. You can create customizations for your Amazon
    (
        "CreateAccountCustomization",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Quick Sight account, or subscribes to Amazon Quick Sight Q. The Amazon Web Services Region for the account is derived from what is configured in the CLI or SDK. Before you use this operation, make sure that you can connect to an existing Amazon Web Services account. If you don't have an Amazon Web Services account, see Sign up for Amazon Web Services in the Amazon Quick Sight Use
    (
        "CreateAccountSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an action connector that enables Amazon Quick Sight to connect to external services and perform actions. Action connectors support various authentication methods and can be configured with specific actions from supported connector types like Amazon S3, Salesforce, JIRA.
    (
        "CreateActionConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an agent in Amazon QuickSight.
    ("CreateAgent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an analysis in Amazon Quick Sight. Analyses can be created either from a template or from an AnalysisDefinition .
    ("CreateAnalysis", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an approval policy in Quick Sight.
    (
        "CreateApprovalPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Quick Sight brand.
    ("CreateBrand", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a custom permissions profile.
    (
        "CreateCustomPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a dashboard from either a template or directly with a DashboardDefinition . To first create a template, see the CreateTemplate API operation. A dashboard is an entity in Amazon Quick Sight that identifies Amazon Quick Sight reports, created from analyses. You can share Amazon Quick Sight dashboards. With the right permissions, you can create scheduled email reports from them. If you have t
    ("CreateDashboard", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a dataset.
    ("CreateDataSet", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a data source.
    ("CreateDataSource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a data loss prevention (DLP) setting configuration for an Amazon Web Services account. A DLP setting defines the DLP provider, the enforcement behavior, and the Quick capabilities that the setting applies to.
    ("CreateDlpSetting", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new flow in the specified Amazon Web Services account. Creates both a DRAFT and PUBLISHED (auto-published) version. This operation is idempotent. Supply a ClientToken to safely retry without creating duplicate resources.
    ("CreateFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an empty shared folder.
    ("CreateFolder", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds an asset, such as a dashboard, analysis, or dataset into a folder.
    (
        "CreateFolderMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use the CreateGroup operation to create a group in Quick Sight. You can create up to 10,000 groups in a namespace. If you want to create more than 10,000 groups in a namespace, contact Amazon Web Services Support. The permissions resource is arn:aws:quicksight: : :group/default/ . The response is a group object.
    ("CreateGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds an Amazon Quick Sight user to an Amazon Quick Sight group.
    (
        "CreateGroupMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an assignment with one specified IAM policy, identified by its Amazon Resource Name (ARN). This policy assignment is attached to the specified groups or users of Amazon Quick Sight. Assignment names are unique per Amazon Web Services account. To avoid overwriting rules in other namespaces, use assignment names that are unique.
    (
        "CreateIAMPolicyAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates and starts a new SPICE ingestion for a dataset. You can manually refresh datasets in an Enterprise edition account 32 times in a 24-hour period. You can manually refresh datasets in a Standard edition account 8 times in a 24-hour period. Each 24-hour period is measured starting 24 hours before the current date and time. Any ingestions operating on tagged datasets inherit the same tags auto
    ("CreateIngestion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a knowledge base from a specified data source. Supported data source connector types include: S3_KNOWLEDGE_BASE – Uses an Amazon S3 bucket as the data source. WEB_CRAWLER – Uses web pages indexed by the built-in web crawler as the data source. GOOGLE_DRIVE – Uses Google Drive as the data source. Supports service account authentication only. SHAREPOINT – Uses SharePoint as the data source.
    (
        "CreateKnowledgeBase",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a limits profile that defines resource usage limits for Amazon Quick Sight users.
    (
        "CreateLimitsProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: (Enterprise edition only) Creates a new namespace for you to use with Amazon Quick Sight. A namespace allows you to isolate the Quick Sight users and groups that are registered for that namespace. Users that access the namespace can share assets only with other users or groups in the same namespace. They can't see users and groups in other namespaces. You can create a namespace after your Amazon W
    ("CreateNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an OAuthClientApplication.
    (
        "CreateOAuthClientApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a refresh schedule for a dataset. You can create up to 5 different schedules for a single dataset.
    (
        "CreateRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use CreateRoleMembership to add an existing Quick Sight group to an existing role.
    (
        "CreateRoleMembership",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon QuickSight space. A space is a collection of resources that can be used to organize and manage QuickSight assets.
    ("CreateSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a template either from a TemplateDefinition or from an existing Quick Sight analysis or template. You can use the resulting template to create additional dashboards, templates, or analyses. A template is an entity in Quick Sight that encapsulates the metadata required to create an analysis and that you can use to create s dashboard. A template adds a layer of abstraction by using placehold
    ("CreateTemplate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a template alias for a template.
    (
        "CreateTemplateAlias",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a theme. A theme is set of configuration options for color and layout. Themes apply to analyses and dashboards. For more information, see Using Themes in Amazon Quick Sight in the Amazon Quick Sight User Guide .
    ("CreateTheme", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a theme alias for a theme.
    ("CreateThemeAlias", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Q topic.
    ("CreateTopic", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a topic refresh schedule.
    (
        "CreateTopicRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Q topic.
    ("CreateTopicV2", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new VPC connection.
    (
        "CreateVPCConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Unapplies a custom permissions profile from an account.
    (
        "DeleteAccountCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This API permanently deletes all Quick Sight customizations for the specified Amazon Web Services account and namespace. When you delete account customizations: All customizations are removed including themes, branding, and visual settings This action cannot be undone through the API Users will see default Quick Sight styling after customizations are deleted Before proceeding: Ensure you have back
    (
        "DeleteAccountCustomization",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deleting your Quick Sight account subscription has permanent, irreversible consequences across all Amazon Web Services regions: Global deletion – Running this operation from any single region will delete your Quick Sight account and all data in every Amazon Web Services region where you have Quick Sight resources. Complete data loss – All dashboards, analyses, datasets, data sources, and custom vi
    (
        "DeleteAccountSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Hard deletes an action connector, making it unrecoverable. This operation removes the connector and all its associated configurations. Any resources currently using this action connector will no longer be able to perform actions through it.
    (
        "DeleteActionConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an agent.
    ("DeleteAgent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an analysis from Amazon Quick Sight. You can optionally include a recovery window during which you can restore the analysis. If you don't specify a recovery window value, the operation defaults to 30 days. Amazon Quick Sight attaches a DeletionTime stamp to the response that specifies the end of the recovery window. At the end of the recovery window, Amazon Quick Sight deletes the analysis
    ("DeleteAnalysis", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an app.
    ("DeleteApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an approval policy in Quick Sight.
    (
        "DeleteApprovalPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This API permanently deletes the specified Quick Sight brand. When you delete a brand: The brand and all its associated branding elements are permanently removed Any applications or dashboards using this brand will revert to default styling This action cannot be undone through the API Before proceeding: Verify that the brand is no longer needed and consider the impact on any applications currently
    ("DeleteBrand", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a brand assignment.
    (
        "DeleteBrandAssignment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a custom permissions profile.
    (
        "DeleteCustomPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a dashboard.
    ("DeleteDashboard", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a dataset.
    ("DeleteDataSet", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the dataset refresh properties of the dataset.
    (
        "DeleteDataSetRefreshProperties",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the data source permanently. This operation breaks all the datasets that reference the deleted data source.
    ("DeleteDataSource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a linked Amazon Q Business application from an Quick Sight account
    (
        "DeleteDefaultQBusinessApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
];
