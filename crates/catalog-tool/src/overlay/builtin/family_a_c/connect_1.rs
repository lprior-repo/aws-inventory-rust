// Pinned source: .local/aws-models/models/connect/service/2017-08-08/connect-2017-08-08.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListContactFlowModuleAliases",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all aliases associated with a contact flow module, showing their current version mappings and metadata.",
            ),
        ),
        (
            "ListContactFlowModuleVersions",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a paginated list of all versions for a specific contact flow module.",
            ),
        ),
        // Model: Provides information about the flow modules for the specified Connect Customer instance.
        (
            "ListContactFlowModules",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "ContactFlowModulesSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListContactFlowVersions",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all the available versions for the specified Connect Customer instance and flow identifier.",
            ),
        ),
        // Model: Provides information about the flows for the specified Connect Customer instance. You can also create and update flows using the Connect Customer Flow language . For more information about flows, see…
        (
            "ListContactFlows",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "ContactFlowSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListContactReferences",
            deferred(
                "Model requires InstanceId, ContactId, ReferenceTypes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. For the specified referenceTypes , returns a list of references associated with the contact. References are links to documents that are related to a…",
            ),
        ),
        (
            "ListDataTableAttributes",
            deferred(
                "Model requires InstanceId, DataTableId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all attributes for a specified data table. A maximum of 100 attributes per data table is allowed. Customers can request an increase by using Amazon Web Services Service Quotas. The response can be filtered by specific…",
            ),
        ),
        // Model: Lists all primary value combinations for a given data table. Returns the unique combinations of primary attribute values that identify records in the table. Up to 100 records are returned per request.
        ("ListDataTablePrimaryValues", denied(DenyReason::Content)),
        // Model: Lists values stored in a data table with optional filtering by record IDs or primary attribute values. Returns the raw stored values along with metadata such as lock versions and modification…
        ("ListDataTableValues", denied(DenyReason::Content)),
        (
            "ListDataTables",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all data tables for the specified Amazon Connect instance. Returns summary information for each table including basic metadata and modification details.",
            ),
        ),
        (
            "ListDefaultVocabularies",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the default vocabularies for the specified Connect Customer instance.",
            ),
        ),
        (
            "ListEntitySecurityProfiles",
            deferred(
                "Model requires InstanceId, EntityType, EntityArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all security profiles attached to a Q in Connect AIAgent Entity in an Amazon Connect instance.",
            ),
        ),
        (
            "ListEvaluationFormAIVersions",
            deferred(
                "Model requires InstanceId, ContactInteractionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the available AI versions for evaluation forms in the specified Connect Customer instance.",
            ),
        ),
        (
            "ListEvaluationFormVersions",
            deferred(
                "Model requires InstanceId, EvaluationFormId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists versions of an evaluation form in the specified Connect Customer instance.",
            ),
        ),
        (
            "ListEvaluationForms",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists evaluation forms in the specified Connect Customer instance.",
            ),
        ),
        (
            "ListExtractionDefinitions",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists extraction definitions in the specified Connect Customer instance.",
            ),
        ),
        (
            "ListFlowAssociations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the flow association based on the filters.",
            ),
        ),
        (
            "ListHoursOfOperationOverrides",
            deferred(
                "Model requires InstanceId, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the hours of operation overrides.",
            ),
        ),
        // Model: Provides information about the hours of operation for the specified Connect Customer instance. For more information about hours of operation, see Set the Hours of Operation for a Queue in the Connect…
        (
            "ListHoursOfOperations",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "HoursOfOperationSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListInstanceAttributes",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Returns a paginated list of all attribute types for the given instance.",
            ),
        ),
        (
            "ListInstanceStorageConfigs",
            deferred(
                "Model requires InstanceId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Returns a paginated list of storage configs for the identified instance and resource type.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: This API is in preview release for Connect Customer and is subject to change. Return a list of instances which are in active state, creation-in-progress state, and failed state. Instances that aren't…
        (
            "ListInstances",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "InstanceSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListIntegrationAssociations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides summary information about the Amazon Web Services resource associations for the specified Connect Customer instance.",
            ),
        ),
        (
            "ListLambdaFunctions",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Returns a paginated list of all Lambda functions that display in the dropdown options in the relevant flow blocks.",
            ),
        ),
        (
            "ListLexBots",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Returns a paginated list of all the Amazon Lex V1 bots currently associated with the instance. To return both Amazon Lex V1 and V2 bots, use the…",
            ),
        ),
        (
            "ListMetrics",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a paginated list of metric summaries for the specified Connect Customer instance. Use pagination to ensure that the operation returns quickly and successfully.",
            ),
        ),
        (
            "ListNotifications",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a paginated list of all notifications in the Amazon Connect instance.",
            ),
        ),
        (
            "ListPhoneNumbers",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides information about the phone numbers for the specified Connect Customer instance. For more information about phone numbers, see Set Up Phone Numbers for Your Contact Center in the Connect Customer Administrator Guide . We…",
            ),
        ),
        // Model: Lists phone numbers claimed to your Connect Customer instance or traffic distribution group. If the provided TargetArn is a traffic distribution group, you can call this API in both Amazon Web…
        (
            "ListPhoneNumbersV2",
            OperationOverlay {
                queryable: vec![path("PhoneNumberArn")?],
                ..read(
                    "ListPhoneNumbersSummaryList",
                    Identity::Arn(path("PhoneNumberArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListPredefinedAttributes",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists predefined attributes for the specified Connect Customer instance. A predefined attribute is made up of a name and a value. You can use predefined attributes for: Routing proficiency (for example, agent certification) that…",
            ),
        ),
        // Model: Provides information about the prompts for the specified Connect Customer instance.
        (
            "ListPrompts",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "PromptSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListQueueEmailAddresses",
            deferred(
                "Model requires InstanceId, QueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all email addresses that are currently associated with a specific queue, providing details about which \"From\" email addresses agents can select when handling email contacts. This helps administrators manage agent email…",
            ),
        ),
        (
            "ListQueueQuickConnects",
            deferred(
                "Model requires InstanceId, QueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the quick connects associated with a queue.",
            ),
        ),
        // Model: Provides information about the queues for the specified Connect Customer instance. If you do not specify a QueueTypes parameter, both standard and agent queues are returned. This might cause an…
        (
            "ListQueues",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "QueueSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListQuickConnects",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "QuickConnectSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        // Model: Provides a list of analysis segments for a real-time chat analysis session. This API supports CHAT channels only. This API does not support VOICE. If you attempt to use it for VOICE, an…
        (
            "ListRealtimeContactAnalysisSegmentsV2",
            denied(DenyReason::Content),
        ),
        (
            "ListReplicationFindings",
            deferred(
                "Model requires InstanceId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists replication findings for resources of a specified resource type in a specified Connect Customer instance. A replication finding describes a single resource whose replication to one or more Regions is incomplete or has…",
            ),
        ),
        (
            "ListRoutingProfileManualAssignmentQueues",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the manual assignment queues associated with a routing profile. Use cases Following are common uses cases for this API: This API returns list of queues where contacts can be manually assigned or picked by an agent who has…",
            ),
        ),
        (
            "ListRoutingProfileQueues",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the queues associated with a routing profile.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Provides summary information about the routing profiles for the specified Connect Customer instance. For more information about routing profiles, see Routing Profiles and Create a Routing Profile in…
        (
            "ListRoutingProfiles",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "RoutingProfileSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListRules",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List all rules for the specified Connect Customer instance.",
            ),
        ),
        (
            "ListSecurityKeys",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Returns a paginated list of all security keys associated with the instance.",
            ),
        ),
        (
            "ListSecurityProfileAIAgents",
            deferred(
                "Model requires SecurityProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the allowed AI agents in a specific security profile.",
            ),
        ),
        (
            "ListSecurityProfileApplications",
            deferred(
                "Model requires SecurityProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of third-party applications or MCP Servers in a specific security profile.",
            ),
        ),
        (
            "ListSecurityProfileFlowModules",
            deferred(
                "Model requires SecurityProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A list of Flow Modules an AI Agent can invoke as a tool",
            ),
        ),
        (
            "ListSecurityProfilePermissions",
            deferred(
                "Model requires SecurityProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the permissions granted to a security profile. For information about security profiles, see Security Profiles in the Connect Customer Administrator Guide . For a mapping of the API name and user interface name of the…",
            ),
        ),
        // Model: Provides summary information about the security profiles for the specified Connect Customer instance. For more information about security profiles, see Security Profiles in the Connect Customer…
        (
            "ListSecurityProfiles",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "SecurityProfileSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags for the specified resource. For sample policies that use tags, see Connect Customer Identity-Based Policy Examples in the Connect Customer Administrator Guide .",
            ),
        ),
        (
            "ListTaskTemplates",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists task templates for the specified Connect Customer instance.",
            ),
        ),
        (
            "ListTestCaseExecutionRecords",
            deferred(
                "Model requires InstanceId, TestCaseId, TestCaseExecutionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists detailed steps of test case execution that includes all observations along with actions taken and data associated in the specified Amazon Connect instance.",
            ),
        ),
        (
            "ListTestCaseExecutions",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all test case executions and allows filtering by test case id, test case name, start time, end time or status of the execution for the specified Amazon Connect instance.",
            ),
        ),
        (
            "ListTestCases",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the test cases present in the specific Amazon Connect instance.",
            ),
        ),
        (
            "ListTrafficDistributionGroupUsers",
            deferred(
                "Model requires TrafficDistributionGroupId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists traffic distribution group users.",
            ),
        ),
        // Model: Lists traffic distribution groups.
        (
            "ListTrafficDistributionGroups",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "TrafficDistributionGroupSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListUseCases",
            deferred(
                "Model requires InstanceId, IntegrationAssociationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the use cases for the integration association.",
            ),
        ),
        (
            "ListUserHierarchyGroups",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "UserHierarchyGroupSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListUserNotifications",
            deferred(
                "Model requires InstanceId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a paginated list of notifications for a specific user, including the notification status for that user.",
            ),
        ),
        (
            "ListUserProficiencies",
            deferred(
                "Model requires InstanceId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists proficiencies associated with a user.",
            ),
        ),
        // UserSummary is identity/username/modification metadata, not profile attributes or credentials.
        (
            "ListUsers",
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
                queryable: vec![path("Arn")?],
                ..read(
                    "UserSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListViewVersions",
            deferred(
                "Model requires InstanceId, ViewId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all the available versions for the specified Connect Customer instance and view identifier. Results will be sorted from highest to lowest.",
            ),
        ),
        // ViewSummary has metadata only; exclude AWS-managed references and never retrieve view content.
        (
            "ListViews",
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
                    path("Type")?,
                    FixedValue::String("CUSTOMER_MANAGED".to_owned()),
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Arn")?],
                ..read(
                    "ViewsSummaryList",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListWorkspaceMedia",
            deferred(
                "Model requires InstanceId, WorkspaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists media assets (such as logos) associated with a workspace.",
            ),
        ),
        (
            "ListWorkspacePages",
            deferred(
                "Model requires InstanceId, WorkspaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the page configurations in a workspace, including the views assigned to each page.",
            ),
        ),
        (
            "ListWorkspaces",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the workspaces in an Amazon Connect instance.",
            ),
        ),
        (
            "MonitorContact",
            deferred(
                "Model requires InstanceId, ContactId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates silent monitoring of a contact. The Contact Control Panel (CCP) of the user specified by userId will be set to silent monitoring mode on the contact.",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PauseContact",
            deferred(
                "Model requires ContactId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Pauses an ongoing task or email contact.",
            ),
        ),
        (
            "PutUserStatus",
            deferred(
                "Model requires UserId, InstanceId, AgentStatusId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes the current status of a user or agent in Connect Customer. If the agent is currently handling a contact, this sets the agent's next status. For more information, see Agent status and Set your next status in the Connect…",
            ),
        ),
        (
            "ReleasePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Releases a phone number previously claimed to an Connect Customer instance or traffic distribution group. You can call this API only in the Amazon Web Services Region where the number was claimed. To release phone numbers from a…",
            ),
        ),
        (
            "ReplicateInstance",
            deferred(
                "Model requires InstanceId, ReplicaRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Replicates an Connect Customer instance in the specified Amazon Web Services Region and copies configuration information for Connect Customer resources across Amazon Web Services Regions. For more information about replicating an…",
            ),
        ),
        (
            "ResumeContact",
            deferred(
                "Model requires ContactId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resumes a task or email contact in a paused state.",
            ),
        ),
        (
            "ResumeContactRecording",
            deferred(
                "Model requires InstanceId, ContactId, InitialContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: When a contact is being recorded, and the recording has been suspended using SuspendContactRecording, this API resumes recording whatever recording is selected in the flow configuration: call, screen, or both. If only call…",
            ),
        ),
        (
            "SearchAgentStatuses",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches AgentStatuses in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchAvailablePhoneNumbers",
            deferred(
                "Model requires PhoneNumberCountryCode, PhoneNumberType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for available phone numbers that you can claim to your Connect Customer instance or traffic distribution group. If the provided TargetArn is a traffic distribution group, you can call this API in both Amazon Web Services…",
            ),
        ),
        (
            "SearchContactEvaluations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches contact evaluations in an Connect Customer instance, with optional filtering. Use cases Following are common uses cases for this API: Find contact evaluations by using specific search criteria. Find contact evaluations…",
            ),
        ),
        (
            "SearchContactFlowModules",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches the flow modules in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchContactFlows",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches the flows in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchContacts",
            deferred(
                "Model requires InstanceId, TimeRange. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches contacts in an Connect Customer instance.",
            ),
        ),
        (
            "SearchDataTables",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for data tables based on the table's ID, name, and description. In the future, this operation can support searching on attribute names and possibly primary values. Follows other search operations closely and supports…",
            ),
        ),
        (
            "SearchEmailAddresses",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches email address in an instance, with optional filtering.",
            ),
        ),
        (
            "SearchEvaluationForms",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches evaluation forms in an Connect Customer instance, with optional filtering. Use cases Following are common uses cases for this API: List all evaluation forms in an instance. Find all evaluation forms that meet specific…",
            ),
        ),
        (
            "SearchHoursOfOperationOverrides",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches the hours of operation overrides.",
            ),
        ),
        (
            "SearchHoursOfOperations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches the hours of operation in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchMetrics",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for metrics in the specified Connect Customer instance using search criteria and optional tag-based filters. Use pagination to ensure that the operation returns quickly and successfully.",
            ),
        ),
        (
            "SearchNotifications",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for notifications based on specified criteria and filters. Returns a paginated list of notifications matching the search parameters, ordered by descending creation time. Supports filtering by content and tags.",
            ),
        ),
        (
            "SearchPredefinedAttributes",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches predefined attributes that meet certain criteria. A predefined attribute is made up of a name and a value. You can use predefined attributes for: Routing proficiency (for example, agent certification) that has predefined…",
            ),
        ),
        (
            "SearchPrompts",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches prompts in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchQueues",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches queues in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchQuickConnects",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches quick connects in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchResourceTags",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches tags used in an Connect Customer instance using optional search criteria.",
            ),
        ),
        (
            "SearchRoutingProfiles",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches routing profiles in an Connect Customer instance, with optional filtering. SearchRoutingProfiles does not populate LastModifiedRegion, LastModifiedTime, MediaConcurrencies.CrossChannelBehavior, and AgentAvailabilityTimer…",
            ),
        ),
        (
            "SearchRules",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches rules in an Connect Customer instance, with optional filtering.",
            ),
        ),
        (
            "SearchSecurityProfiles",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches security profiles in an Connect Customer instance, with optional filtering. For information about security profiles, see Security Profiles in the Connect Customer Administrator Guide . For a mapping of the API name and…",
            ),
        ),
        (
            "SearchTestCases",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for test cases in the specified Amazon Connect instance, with optional filtering.",
            ),
        ),
        (
            "SearchUserHierarchyGroups",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches UserHierarchyGroups in an Connect Customer instance, with optional filtering. The UserHierarchyGroup with \"LevelId\": \"0\" is the foundation for building levels on top of an instance. It is not user-definable, nor is it…",
            ),
        ),
        (
            "SearchUsers",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches users in an Connect Customer instance, with optional filtering. AfterContactWorkTimeLimit is returned in milliseconds.",
            ),
        ),
        (
            "SearchViews",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches views based on name, description, or tags.",
            ),
        ),
        (
            "SearchVocabularies",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for vocabularies within a specific Connect Customer instance using State , NameStartsWith , and LanguageCode .",
            ),
        ),
        (
            "SearchWorkspaceAssociations",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for workspace associations with users or routing profiles based on various criteria.",
            ),
        ),
        (
            "SearchWorkspaces",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches workspaces based on name, description, visibility, or tags.",
            ),
        ),
        (
            "SendChatIntegrationEvent",
            deferred(
                "Model requires SourceId, DestinationId, Event. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Processes chat integration events from Amazon Web Services or external integrations to Connect Customer. A chat integration event includes: SourceId, DestinationId, and Subtype: a set of identifiers, uniquely representing a chat…",
            ),
        ),
        (
            "SendOutboundEmail",
            deferred(
                "Model requires InstanceId, FromEmailAddress, DestinationEmailAddress, EmailMessage, TrafficType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Send outbound email for outbound campaigns. For more information about outbound campaigns, see Set up Connect Customer outbound campaigns . Only the Connect Customer outbound campaigns service principal is allowed to assume a…",
            ),
        ),
        (
            "SendOutboundWebNotification",
            deferred(
                "Model requires InstanceId, BrowserId, SessionId, ExpiresAt, Source, Destination, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends an outbound web notification to a customer's web browser for outbound campaigns. For more information about outbound campaigns, see Set up Connect Customer outbound campaigns . Only the Connect Customer outbound campaigns…",
            ),
        ),
        (
            "StartAssistantContact",
            deferred(
                "Model requires InstanceId, AiAgent, ParticipantDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a chat contact with an AI agent. Use the returned ParticipantToken with the CreateParticipantConnection operation. For more information about chat, see the following topics in the Connect Customer Administrator Guide :…",
            ),
        ),
        (
            "StartAttachedFileUpload",
            deferred(
                "Model requires InstanceId, FileName, FileSizeInBytes, FileUseCaseType, AssociatedResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides a pre-signed Amazon S3 URL in response for uploading your content. You may only use this API to upload attachments to a Connect Customer Case , Connect Customer Email , or Connect Customer Task .",
            ),
        ),
        (
            "StartChatContact",
            deferred(
                "Model requires InstanceId, ContactFlowId, ParticipantDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a flow to start a new chat for the customer. Response of this API provides a token required to obtain credentials from the CreateParticipantConnection API in the Connect Customer Participant Service. When a new chat…",
            ),
        ),
        (
            "StartContactConversationalAnalyticsJob",
            deferred(
                "Model requires InstanceId, ContactId, AnalyticsModes, AnalyticsConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a Contact Lens post-call analytics job for the specified contact. This API runs Conversational Analytics post-contact analysis on a voice recording that is already attached to the contact, generating transcription,…",
            ),
        ),
        (
            "StartContactEvaluation",
            deferred(
                "Model requires InstanceId, ContactId, EvaluationFormId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an empty evaluation in the specified Connect Customer instance, using the given evaluation form for the particular contact. The evaluation form version used for the contact evaluation corresponds to the currently activated…",
            ),
        ),
    ])
}

pub(super) fn part_4() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
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
