// Pinned source: .local/aws-models/models/connect/service/2017-08-08/connect-2017-08-08.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeDataTableAttribute",
            deferred(
                "Model requires InstanceId, DataTableId, AttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed information for a specific data table attribute including its configuration, validation rules, and metadata. \"Describe\" is a deprecated term but is allowed to maintain consistency with existing operations.",
            ),
        ),
        (
            "DescribeEmailAddress",
            deferred(
                "Model requires InstanceId, EmailAddressId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describe email address form the specified Connect Customer instance.",
            ),
        ),
        (
            "DescribeEvaluationForm",
            deferred(
                "Model requires InstanceId, EvaluationFormId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an evaluation form in the specified Connect Customer instance. If the version property is not provided, the latest version of the evaluation form is described.",
            ),
        ),
        (
            "DescribeExtractionDefinition",
            deferred(
                "Model requires InstanceId, ExtractionDefinitionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an extraction definition in the specified Connect Customer instance.",
            ),
        ),
        (
            "DescribeHoursOfOperation",
            deferred(
                "Model requires InstanceId, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the hours of operation.",
            ),
        ),
        (
            "DescribeHoursOfOperationOverride",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, HoursOfOperationOverrideId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the hours of operation override.",
            ),
        ),
        // Model: This API is in preview release for Connect Customer and is subject to change. Returns the current state of the specified instance identifier. It tracks the instance while it is being created and…
        (
            "DescribeInstance",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("InstanceId")?,
                    Binding::From {
                        operation: name("ListInstances")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("Tags")?),
                queryable: vec![path("Arn")?],
                ..read(
                    "Instance",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeInstanceAttribute",
            deferred(
                "Model requires InstanceId, AttributeType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Describes the specified instance attribute.",
            ),
        ),
        (
            "DescribeInstanceStorageConfig",
            deferred(
                "Model requires InstanceId, AssociationId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Retrieves the current storage configurations for the specified resource type, association ID, and instance ID.",
            ),
        ),
        (
            "DescribeMetric",
            deferred(
                "Model requires InstanceId, MetricId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the full definition of an existing metric from the specified Connect Customer instance.",
            ),
        ),
        (
            "DescribeNotification",
            deferred(
                "Model requires InstanceId, NotificationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific notification, including its content, priority, recipients, and metadata.",
            ),
        ),
        (
            "DescribePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details and status of a phone number that’s claimed to your Connect Customer instance or traffic distribution group. If the number is claimed to a traffic distribution group, and you are calling in the Amazon Web Services…",
            ),
        ),
        (
            "DescribePredefinedAttribute",
            deferred(
                "Model requires InstanceId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a predefined attribute for the specified Connect Customer instance. A predefined attribute is made up of a name and a value. You can use predefined attributes for: Routing proficiency (for example, agent certification)…",
            ),
        ),
        (
            "DescribePrompt",
            deferred(
                "Model requires InstanceId, PromptId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the prompt.",
            ),
        ),
        (
            "DescribeQueue",
            deferred(
                "Model requires InstanceId, QueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified queue.",
            ),
        ),
        (
            "DescribeQuickConnect",
            deferred(
                "Model requires InstanceId, QuickConnectId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the quick connect.",
            ),
        ),
        (
            "DescribeRoutingProfile",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified routing profile. DescribeRoutingProfile does not populate AssociatedQueueIds in its response. The example Response Syntax shown on this page is incorrect; we are working to update it. SearchRoutingProfiles…",
            ),
        ),
        (
            "DescribeRule",
            deferred(
                "Model requires InstanceId, RuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a rule for the specified Connect Customer instance.",
            ),
        ),
        (
            "DescribeSecurityProfile",
            deferred(
                "Model requires SecurityProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets basic information about the security profile. For information about security profiles, see Security Profiles in the Connect Customer Administrator Guide . For a mapping of the API name and user interface name of the security…",
            ),
        ),
        (
            "DescribeTestCase",
            deferred(
                "Model requires InstanceId, TestCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified test case and allows you to get the content and metadata of the test case for the specified Amazon Connect instance.",
            ),
        ),
        (
            "DescribeTrafficDistributionGroup",
            deferred(
                "Model requires TrafficDistributionGroupId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details and status of a traffic distribution group.",
            ),
        ),
        (
            "DescribeUser",
            deferred(
                "Model requires UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified user. You can find the instance ID in the Connect Customer console (it’s the final part of the ARN). The console does not display the user IDs. Instead, list the users and note the IDs provided in the…",
            ),
        ),
        (
            "DescribeUserHierarchyGroup",
            deferred(
                "Model requires HierarchyGroupId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified hierarchy group.",
            ),
        ),
        (
            "DescribeUserHierarchyStructure",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the hierarchy structure of the specified Connect Customer instance.",
            ),
        ),
        (
            "DescribeView",
            deferred(
                "Model requires InstanceId, ViewId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the view for the specified Connect Customer instance and view identifier. The view identifier can be supplied as a ViewId or ARN. $SAVED needs to be supplied if a view is unpublished. The view identifier can contain an…",
            ),
        ),
        (
            "DescribeVocabulary",
            deferred(
                "Model requires InstanceId, VocabularyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified vocabulary.",
            ),
        ),
        (
            "DescribeWorkspace",
            deferred(
                "Model requires InstanceId, WorkspaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a workspace, including its configuration and metadata.",
            ),
        ),
        (
            "DisassociateAnalyticsDataSet",
            deferred(
                "Model requires InstanceId, DataSetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the dataset ID associated with a given Connect Customer instance.",
            ),
        ),
        (
            "DisassociateApprovedOrigin",
            deferred(
                "Model requires InstanceId, Origin. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Revokes access to integrated applications from Connect Customer.",
            ),
        ),
        (
            "DisassociateBot",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Revokes authorization from the specified instance to access the specified Amazon Lex or Amazon Lex V2 bot.",
            ),
        ),
        (
            "DisassociateEmailAddressAlias",
            deferred(
                "Model requires EmailAddressId, InstanceId, AliasConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the alias association between two email addresses in an Connect Customer instance. After disassociation, emails sent to the former alias email address are no longer forwarded to the primary email address. Both email…",
            ),
        ),
        (
            "DisassociateFlow",
            deferred(
                "Model requires InstanceId, ResourceId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a connect resource from a flow.",
            ),
        ),
        (
            "DisassociateHoursOfOperations",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, ParentHoursOfOperationIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a set of hours of operations with another hours of operation. For more information about inheriting overrides from parent hours of operation, see Hours of operation overrides in the Administrator Guide.",
            ),
        ),
        (
            "DisassociateInstanceStorageConfig",
            deferred(
                "Model requires InstanceId, AssociationId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Removes the storage type configurations for the specified resource type and association ID.",
            ),
        ),
        (
            "DisassociateLambdaFunction",
            deferred(
                "Model requires InstanceId, FunctionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Remove the Lambda function from the dropdown options available in the relevant flow blocks.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DisassociateLexBot",
            deferred(
                "Model requires InstanceId, BotName, LexRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Revokes authorization from the specified instance to access the specified Amazon Lex bot.",
            ),
        ),
        (
            "DisassociatePhoneNumberContactFlow",
            deferred(
                "Model requires PhoneNumberId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the flow association from a phone number claimed to your Connect Customer instance. If the number is claimed to a traffic distribution group, and you are calling this API using an instance in the Amazon Web Services…",
            ),
        ),
        (
            "DisassociateQueueEmailAddresses",
            deferred(
                "Model requires InstanceId, QueueId, EmailAddressesId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between a set of email addresses and a queue. After disassociation, agents will no longer be able to select these email addresses as \"From\" addresses when replying to inbound email contacts or initiating…",
            ),
        ),
        (
            "DisassociateQueueQuickConnects",
            deferred(
                "Model requires InstanceId, QueueId, QuickConnectIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a set of quick connects from a queue.",
            ),
        ),
        (
            "DisassociateRoutingProfileQueues",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a set of queues from a routing profile. Up to 10 queue references can be disassociated in a single API call. More than 10 queue references results in a single call results in an InvalidParameterException.",
            ),
        ),
        (
            "DisassociateSecurityKey",
            deferred(
                "Model requires InstanceId, AssociationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Deletes the specified security key.",
            ),
        ),
        (
            "DisassociateSecurityProfiles",
            deferred(
                "Model requires InstanceId, SecurityProfiles, EntityType, EntityArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a security profile attached to a Q in Connect AI Agent Entity in an Amazon Connect instance.",
            ),
        ),
        (
            "DisassociateTrafficDistributionGroupUser",
            deferred(
                "Model requires TrafficDistributionGroupId, UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates an agent from a traffic distribution group. This API can be called only in the Region where the traffic distribution group is created.",
            ),
        ),
        (
            "DisassociateUserProficiencies",
            deferred(
                "Model requires InstanceId, UserId, UserProficiencies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a set of proficiencies from a user.",
            ),
        ),
        (
            "DisassociateWorkspace",
            deferred(
                "Model requires InstanceId, WorkspaceId, ResourceArns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between a workspace and one or more users or routing profiles.",
            ),
        ),
        (
            "DismissUserContact",
            deferred(
                "Model requires UserId, InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Dismisses contacts from an agent’s CCP and returns the agent to an available state, which allows the agent to receive a new routed contact. Contacts can only be dismissed if they are in a MISSED , ERROR , ENDED , or REJECTED…",
            ),
        ),
        (
            "EvaluateDataTableValues",
            deferred(
                "Model requires InstanceId, DataTableId, Values. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Evaluates values at the time of the request and returns them. It considers the request's timezone or the table's timezone, in that order, when accessing time based tables. When a value is accessed, the accessor's identity and the…",
            ),
        ),
        (
            "GetAttachedFile",
            deferred(
                "Model requires InstanceId, FileId, AssociatedResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides a pre-signed URL for download of an approved attached file. This API also returns metadata about the attached file. It will only return a downloadURL if the status of the attached file is APPROVED .",
            ),
        ),
        (
            "GetContactAttributes",
            deferred(
                "Model requires InstanceId, InitialContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the contact attributes for the specified contact.",
            ),
        ),
        (
            "GetContactMetrics",
            deferred(
                "Model requires InstanceId, ContactId, Metrics. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves contact metric data for a specified contact. Use cases Following are common use cases for position in queue and estimated wait time: Customer-Facing Wait Time Announcements - Display or announce the estimated wait time…",
            ),
        ),
        (
            "GetCrossRegionRouting",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current cross-region routing configuration for an Amazon Connect Global Resiliency instance enabled for global routing. This operation returns whether cross-region routing is currently enabled or disabled (isolated)…",
            ),
        ),
        (
            "GetCurrentMetricData",
            deferred(
                "Model requires InstanceId, Filters, CurrentMetrics. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the real-time metric data from the specified Connect Customer instance. For a description of each metric, see Metrics definitions in the Connect Customer Administrator Guide . When you make a successful API request, you can…",
            ),
        ),
        (
            "GetCurrentUserData",
            deferred(
                "Model requires InstanceId, Filters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the real-time active user data from the specified Connect Customer instance.",
            ),
        ),
        (
            "GetEffectiveHoursOfOperations",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, FromDate, ToDate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the hours of operations with the effective override applied.",
            ),
        ),
        (
            "GetEvaluationFormValidation",
            deferred(
                "Model requires InstanceId, EvaluationFormId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the status and results of a validation process started by StartEvaluationFormValidation . Returns the current execution status ( IN_PROGRESS , COMPLETED , or FAILED ), the validated form version, and when completed, a…",
            ),
        ),
        (
            "GetFederationToken",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Supports SAML sign-in for Connect Customer. Retrieves a token for federation. The token is for the Connect Customer user which corresponds to the IAM credentials that were used to invoke this action. For more information about…",
            ),
        ),
        (
            "GetFlowAssociation",
            deferred(
                "Model requires InstanceId, ResourceId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the flow associated for a given resource.",
            ),
        ),
        // Model: Gets historical metric data from the specified Connect Customer instance. For a description of each historical metric, see Metrics definitions in the Connect Customer Administrator Guide . We…
        ("GetMetricData", denied(DenyReason::Paid)),
        (
            "GetMetricDataV2",
            deferred(
                "Model requires ResourceArn, StartTime, EndTime, Filters, Metrics. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets metric data from the specified Connect Customer instance. GetMetricDataV2 offers more features than GetMetricData , the previous version of this API. It has new metrics, offers filtering at a metric level, and offers the…",
            ),
        ),
        (
            "GetPromptFile",
            deferred(
                "Model requires InstanceId, PromptId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the prompt file.",
            ),
        ),
        (
            "GetTaskTemplate",
            deferred(
                "Model requires InstanceId, TaskTemplateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details about a specific task template in the specified Connect Customer instance.",
            ),
        ),
        (
            "GetTestCaseExecutionSummary",
            deferred(
                "Model requires InstanceId, TestCaseId, TestCaseExecutionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an overview of a test execution that includes the status of the execution, start and end time, and observation summary.",
            ),
        ),
        (
            "GetTrafficDistribution",
            deferred(
                "Model requires Id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current traffic distribution for a given traffic distribution group.",
            ),
        ),
        (
            "ImportPhoneNumber",
            deferred(
                "Model requires InstanceId, SourcePhoneNumberArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports a claimed phone number from an external service, such as Amazon Web Services End User Messaging, into an Connect Customer instance. You can call this API only in the same Amazon Web Services Region where the Connect…",
            ),
        ),
        (
            "ImportWorkspaceMedia",
            deferred(
                "Model requires InstanceId, WorkspaceId, MediaType, MediaSource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports a media asset (such as a logo) for use in a workspace.",
            ),
        ),
        // Exclude built-in ROUTABLE/OFFLINE references at the request boundary.
        (
            "ListAgentStatuses",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("InstanceId")?,
                    Binding::From {
                        operation: name("ListInstances")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                fixed: [(
                    path("AgentStatusTypes")?,
                    FixedValue::List(vec![FixedValue::String("CUSTOM".to_owned())]),
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Arn")?],
                ..read(
                    "AgentStatusSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListAnalyticsDataAssociations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the association status of requested dataset ID for a given Connect Customer instance.",
            ),
        ),
        (
            "ListAnalyticsDataLakeDataSets",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the data lake datasets available to associate with for a given Connect Customer instance.",
            ),
        ),
        (
            "ListApprovedOrigins",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Returns a paginated list of all approved origins associated with the instance.",
            ),
        ),
        (
            "ListAssociatedContacts",
            deferred(
                "Model requires InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides information about contact tree, a list of associated contacts with a unique identifier.",
            ),
        ),
        (
            "ListAttachedFilesConfigurations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides summary information about the attached files configurations for the specified Connect Customer instance. This API returns effective configurations (custom overrides or defaults) for each attachment scope. If no custom…",
            ),
        ),
        (
            "ListAuthenticationProfiles",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. To request access to this API, contact Amazon Web Services Support. Provides summary information about the authentication profiles in a specified…",
            ),
        ),
        (
            "ListBots",
            deferred(
                "Model requires InstanceId, LexVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. For the specified version of Amazon Lex, returns a paginated list of all the Amazon Lex bots currently associated with the instance. Use this API to…",
            ),
        ),
        (
            "ListChildHoursOfOperations",
            deferred(
                "Model requires InstanceId, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides information about the child hours of operations for the specified parent hours of operation. For more information about child hours of operations, see Link overrides from different hours of operation in the Administrator…",
            ),
        ),
        (
            "ListContactEvaluations",
            deferred(
                "Model requires InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists contact evaluations in the specified Connect Customer instance.",
            ),
        ),
        (
            "ListContactFlowAliases",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the aliases associated with a flow, including the version that each alias points to.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "StartContactMediaProcessing",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Enables in-flight message processing for an ongoing chat session. Message processing will stay active for the rest of the chat, even if an individual contact segment ends.",
            ),
        ),
        (
            "StartContactRecording",
            deferred(
                "Model requires InstanceId, ContactId, InitialContactId, VoiceRecordingConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts recording the contact: If the API is called before the agent joins the call, recording starts when the agent joins the call. If the API is called after the agent joins the call, recording starts at the time of the API…",
            ),
        ),
        (
            "StartContactStreaming",
            deferred(
                "Model requires InstanceId, ContactId, ChatStreamingConfiguration, ClientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates real-time message streaming for a new chat contact. For more information about message streaming, see Enable real-time chat message streaming in the Connect Customer Administrator Guide . For more information about…",
            ),
        ),
        (
            "StartEmailContact",
            deferred(
                "Model requires InstanceId, FromEmailAddress, DestinationEmailAddress, EmailMessage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an inbound email contact and initiates a flow to start the email contact for the customer. Response of this API provides the ContactId of the email contact created.",
            ),
        ),
        (
            "StartEvaluationFormValidation",
            deferred(
                "Model requires InstanceId, EvaluationFormId, EvaluationFormVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an asynchronous validation process for an evaluation form version in the specified Connect Customer instance. The validation first performs structural checks on the form content (such as verifying required fields, valid…",
            ),
        ),
        (
            "StartOutboundChatContact",
            deferred(
                "Model requires SourceEndpoint, DestinationEndpoint, InstanceId, SegmentAttributes, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a new outbound SMS or WhatsApp contact to a customer. Response of this API provides the ContactId of the outbound SMS or WhatsApp contact created. SourceEndpoint only supports Endpoints with CONNECT_PHONENUMBER_ARN as…",
            ),
        ),
        (
            "StartOutboundEmailContact",
            deferred(
                "Model requires InstanceId, ContactId, DestinationEmailAddress, EmailMessage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a flow to send an agent reply or outbound email contact (created from the CreateContact API) to a customer.",
            ),
        ),
        (
            "StartOutboundVoiceContact",
            deferred(
                "Model requires DestinationPhoneNumber, ContactFlowId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Places an outbound call to a contact, and then initiates the flow. It performs the actions in the flow that's specified (in ContactFlowId ). Agents do not initiate the outbound API, which means that they do not dial the contact.…",
            ),
        ),
        (
            "StartScreenSharing",
            deferred(
                "Model requires InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts screen sharing for a contact. For more information about screen sharing, see Set up in-app, web, video calling, and screen sharing capabilities in the Connect Customer Administrator Guide .",
            ),
        ),
        (
            "StartTaskContact",
            deferred(
                "Model requires InstanceId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a flow to start a new task contact. For more information about task contacts, see Concepts: Tasks in Connect Customer in the Connect Customer Administrator Guide . When using PreviousContactId and RelatedContactId input…",
            ),
        ),
        (
            "StartTestCaseExecution",
            deferred(
                "Model requires InstanceId, TestCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts executing a published test case.",
            ),
        ),
        (
            "StartWebRTCContact",
            deferred(
                "Model requires ContactFlowId, InstanceId, ParticipantDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Places an inbound in-app, web, or video call to a contact, and then initiates the flow. It performs the actions in the flow that are specified (in ContactFlowId) and present in the Connect Customer instance (specified as…",
            ),
        ),
        (
            "StopContact",
            deferred(
                "Model requires ContactId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ends the specified contact. Use this API to stop queued callbacks. It does not work for voice contacts that use the following initiation methods: DISCONNECT TRANSFER QUEUE_TRANSFER EXTERNAL_OUTBOUND MONITOR Chat and task contacts…",
            ),
        ),
        (
            "StopContactMediaProcessing",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Stops in-flight message processing for an ongoing chat session.",
            ),
        ),
        (
            "StopContactRecording",
            deferred(
                "Model requires InstanceId, ContactId, InitialContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops recording a call when a contact is being recorded. StopContactRecording is a one-time action. If you use StopContactRecording to stop recording an ongoing call, you can't use StartContactRecording to restart it. For…",
            ),
        ),
        (
            "StopContactStreaming",
            deferred(
                "Model requires InstanceId, ContactId, StreamingId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ends message streaming on a specified contact. To restart message streaming on that contact, call the StartContactStreaming API.",
            ),
        ),
        (
            "StopTestCaseExecution",
            deferred(
                "Model requires InstanceId, TestCaseExecutionId, TestCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a running test execution.",
            ),
        ),
        (
            "SubmitContactEvaluation",
            deferred(
                "Model requires InstanceId, EvaluationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits a contact evaluation in the specified Connect Customer instance. Answers included in the request are merged with existing answers for the given evaluation. If no answers or notes are passed, the evaluation is submitted…",
            ),
        ),
        (
            "SuspendContactRecording",
            deferred(
                "Model requires InstanceId, ContactId, InitialContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: When a contact is being recorded, this API suspends recording whatever is selected in the flow configuration: call (IVR or agent), screen, or both. If only call recording or only screen recording is enabled, then it would be…",
            ),
        ),
        (
            "TagContact",
            deferred(
                "Model requires ContactId, InstanceId, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds the specified tags to the contact resource. For more information about this API is used, see Set up granular billing for a detailed view of your Connect Customer usage .",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds the specified tags to the specified resource. Some of the supported resource types are agents, routing profiles, queues, quick connects, flows, agent statuses, hours of operation, phone numbers, security profiles, task…",
            ),
        ),
        (
            "TransferContact",
            deferred(
                "Model requires InstanceId, ContactId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Transfers TASK or EMAIL contacts from one agent or queue to another agent or queue at any point after a contact is created. You can transfer a contact to another queue by providing the flow which orchestrates the contact to the…",
            ),
        ),
        (
            "UntagContact",
            deferred(
                "Model requires ContactId, InstanceId, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the contact resource. For more information about this API is used, see Set up granular billing for a detailed view of your Connect Customer usage .",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified resource.",
            ),
        ),
        (
            "UpdateAgentStatus",
            deferred(
                "Model requires InstanceId, AgentStatusId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates agent status.",
            ),
        ),
        (
            "UpdateAttachedFilesConfiguration",
            deferred(
                "Model requires InstanceId, AttachmentScope. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the attached files configuration for the specified Connect Customer instance and attachment scope. If no instance-specific configuration exists, this operation creates one. Partial updates are supported—only specified…",
            ),
        ),
        (
            "UpdateAuthenticationProfile",
            deferred(
                "Model requires AuthenticationProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. To request access to this API, contact Amazon Web Services Support. Updates the selected authentication profile.",
            ),
        ),
        (
            "UpdateContact",
            deferred(
                "Model requires InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Adds or updates user-defined contact information associated with the specified contact. At least one field to be updated must be present in the…",
            ),
        ),
        (
            "UpdateContactAttributes",
            deferred(
                "Model requires InitialContactId, InstanceId, Attributes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates user-defined contact attributes associated with the specified contact. You can create or update user-defined attributes for both ongoing and completed contacts. For example, while the call is active, you can…",
            ),
        ),
        (
            "UpdateContactEvaluation",
            deferred(
                "Model requires InstanceId, EvaluationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates details about a contact evaluation in the specified Connect Customer instance. A contact evaluation must be in draft state. Answers included in the request are merged with existing answers for the given evaluation. An…",
            ),
        ),
        (
            "UpdateContactFlowAlias",
            deferred(
                "Model requires InstanceId, ContactFlowId, AliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name, description, or flow version associated with an alias.",
            ),
        ),
        (
            "UpdateContactFlowContent",
            deferred(
                "Model requires InstanceId, ContactFlowId, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified flow. You can also create and update flows using the Connect Customer Flow language . Use the $SAVED alias in the request to describe the SAVED content of a Flow. For example,…",
            ),
        ),
        (
            "UpdateContactFlowMetadata",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates metadata about specified flow.",
            ),
        ),
        (
            "UpdateContactFlowModuleAlias",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId, AliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a specific Aliases metadata, including the version it’s tied to, it’s name, and description.",
            ),
        ),
        (
            "UpdateContactFlowModuleContent",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates specified flow module for the specified Connect Customer instance. Use the $SAVED alias in the request to describe the SAVED content of a Flow. For example, arn:aws:.../contact-flow/{id}:$SAVED . After a flow is…",
            ),
        ),
        (
            "UpdateContactFlowModuleMetadata",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates metadata about specified flow module.",
            ),
        ),
        (
            "UpdateContactFlowName",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The name of the flow. You can also create and update flows using the Connect Customer Flow language .",
            ),
        ),
        (
            "UpdateContactRoutingData",
            deferred(
                "Model requires InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates routing priority and age on the contact ( QueuePriority and QueueTimeAdjustmentInSeconds ). These properties can be used to change a customer's position in the queue. For example, you can move a contact to the back of the…",
            ),
        ),
        (
            "UpdateContactSchedule",
            deferred(
                "Model requires InstanceId, ContactId, ScheduledTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the scheduled time of a task contact that is already scheduled.",
            ),
        ),
        (
            "UpdateContactTaskTemplate",
            deferred(
                "Model requires InstanceId, TaskTemplateId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the task template association on an existing task contact. You can update the task template on a contact before assignment to support tasks that are created without a template (for example Rules or disconnect flows ) or…",
            ),
        ),
        (
            "UpdateCrossRegionRouting",
            deferred(
                "Model requires InstanceId, IsolatedAll. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the cross-region routing configuration for an Amazon Connect Global Resiliency instance enabled for global routing. When invoked with IsolatedAll set to true , this operation disables cross-region routing, meaning…",
            ),
        ),
        (
            "UpdateDataTableAttribute",
            deferred(
                "Model requires InstanceId, DataTableId, AttributeName, Name, ValueType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates all properties for an attribute using all properties from CreateDataTableAttribute. There are no other granular update endpoints. It does not act as a patch operation - all properties must be provided. System managed…",
            ),
        ),
    ])
}
