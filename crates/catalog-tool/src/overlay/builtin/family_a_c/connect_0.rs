// Pinned source: .local/aws-models/models/connect/service/2017-08-08/connect-2017-08-08.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ActivateEvaluationForm",
            deferred(
                "Model requires InstanceId, EvaluationFormId, EvaluationFormVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Activates an evaluation form in the specified Connect Customer instance. After the evaluation form is activated, it is available to start new evaluations based on the form.",
            ),
        ),
        (
            "AssociateAnalyticsDataSet",
            deferred(
                "Model requires InstanceId, DataSetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified dataset for a Connect Customer instance with the target account. You can associate only one dataset in a single call.",
            ),
        ),
        (
            "AssociateApprovedOrigin",
            deferred(
                "Model requires InstanceId, Origin. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Associates an approved origin to an Connect Customer instance.",
            ),
        ),
        (
            "AssociateBot",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Allows the specified Connect Customer instance to access the specified Amazon Lex or Amazon Lex V2 bot.",
            ),
        ),
        (
            "AssociateContactWithUser",
            deferred(
                "Model requires InstanceId, ContactId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a queued contact with an agent. Use cases Following are common uses cases for this API: Programmatically assign queued contacts to available users. Leverage the IAM context key connect:PreferredUserArn to restrict…",
            ),
        ),
        (
            "AssociateDefaultVocabulary",
            deferred(
                "Model requires InstanceId, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates an existing vocabulary as the default. Contact Lens for Connect Customer uses the vocabulary in post-call and real-time analysis sessions for the given language.",
            ),
        ),
        (
            "AssociateEmailAddressAlias",
            deferred(
                "Model requires EmailAddressId, InstanceId, AliasConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates an email address alias with an existing email address in an Connect Customer instance. This creates a forwarding relationship where emails sent to the alias email address are automatically forwarded to the primary…",
            ),
        ),
        (
            "AssociateFlow",
            deferred(
                "Model requires InstanceId, ResourceId, FlowId, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a connect resource to a flow.",
            ),
        ),
        (
            "AssociateHoursOfOperations",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, ParentHoursOfOperationConfigs. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of hours of operations with another hours of operation. For more information about inheriting overrides from parent hours of operation, see Hours of operation overrides in the Administrator Guide.",
            ),
        ),
        (
            "AssociateInstanceStorageConfig",
            deferred(
                "Model requires InstanceId, ResourceType, StorageConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Associates a storage resource type for the first time. You can only associate one type of storage configuration in a single call. This means, for…",
            ),
        ),
        (
            "AssociateLambdaFunction",
            deferred(
                "Model requires InstanceId, FunctionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Allows the specified Connect Customer instance to access the specified Lambda function.",
            ),
        ),
        (
            "AssociateLexBot",
            deferred(
                "Model requires InstanceId, LexBot. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Allows the specified Connect Customer instance to access the specified Amazon Lex V1 bot. This API only supports the association of Amazon Lex V1 bots.",
            ),
        ),
        (
            "AssociatePhoneNumberContactFlow",
            deferred(
                "Model requires PhoneNumberId, InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a flow with a phone number claimed to your Connect Customer instance. If the number is claimed to a traffic distribution group, and you are calling this API using an instance in the Amazon Web Services Region where the…",
            ),
        ),
        (
            "AssociateQueueEmailAddresses",
            deferred(
                "Model requires InstanceId, QueueId, EmailAddressesConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of email addresses with a queue to enable agents to select different \"From\" (system) email addresses when replying to inbound email contacts or initiating outbound email contacts. This allows agents to handle…",
            ),
        ),
        (
            "AssociateQueueQuickConnects",
            deferred(
                "Model requires InstanceId, QueueId, QuickConnectIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of quick connects with a queue.",
            ),
        ),
        (
            "AssociateRoutingProfileQueues",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of queues with a routing profile.",
            ),
        ),
        (
            "AssociateSecurityKey",
            deferred(
                "Model requires InstanceId, Key. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Associates a security key to the instance.",
            ),
        ),
        (
            "AssociateSecurityProfiles",
            deferred(
                "Model requires InstanceId, SecurityProfiles, EntityType, EntityArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associate security profiles with an Entity in an Amazon Connect instance.",
            ),
        ),
        (
            "AssociateTrafficDistributionGroupUser",
            deferred(
                "Model requires TrafficDistributionGroupId, UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates an agent with a traffic distribution group. This API can be called only in the Region where the traffic distribution group is created.",
            ),
        ),
        (
            "AssociateUserProficiencies",
            deferred(
                "Model requires InstanceId, UserId, UserProficiencies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a set of proficiencies with a user.",
            ),
        ),
        (
            "AssociateWorkspace",
            deferred(
                "Model requires InstanceId, WorkspaceId, ResourceArns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a workspace with one or more users or routing profiles, allowing them to access the workspace's configured views and pages.",
            ),
        ),
        (
            "BatchAssociateAnalyticsDataSet",
            deferred(
                "Model requires InstanceId, DataSetIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a list of analytics datasets for a given Connect Customer instance to a target account. You can associate multiple datasets in a single call.",
            ),
        ),
        (
            "BatchCreateDataTableValue",
            deferred(
                "Model requires InstanceId, DataTableId, Values. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates values for attributes in a data table. The value may be a default or it may be associated with a primary value. The value must pass all customer defined validation as well as the default validation for the value type. The…",
            ),
        ),
        (
            "BatchDeleteDataTableValue",
            deferred(
                "Model requires InstanceId, DataTableId, Values. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes multiple values from a data table. API users may delete values at any time. When deletion is requested from the admin website, a warning is shown alerting the user of the most recent time the attribute and its values were…",
            ),
        ),
        (
            "BatchDescribeDataTableValue",
            deferred(
                "Model requires InstanceId, DataTableId, Values. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves multiple values from a data table without evaluating expressions. Returns the raw stored values along with metadata such as lock versions and modification timestamps. \"Describe\" is a deprecated term but is allowed to…",
            ),
        ),
        (
            "BatchDisassociateAnalyticsDataSet",
            deferred(
                "Model requires InstanceId, DataSetIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a list of analytics datasets associated with a given Connect Customer instance. You can disassociate multiple datasets in a single call.",
            ),
        ),
        (
            "BatchGetAttachedFileMetadata",
            deferred(
                "Model requires FileIds, InstanceId, AssociatedResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows you to retrieve metadata about multiple attached files on an associated resource. Each attached file provided in the input list must be associated with the input AssociatedResourceArn.",
            ),
        ),
        (
            "BatchGetFlowAssociation",
            deferred(
                "Model requires InstanceId, ResourceIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve the flow associations for the given resources.",
            ),
        ),
        (
            "BatchPutContact",
            deferred(
                "Model requires InstanceId, ContactDataRequestList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Only the Connect Customer outbound campaigns service principal is allowed to assume a role in your account and call this API. Allows you to create a batch of contacts in Connect Customer. The outbound campaigns capability ingests…",
            ),
        ),
        (
            "BatchUpdateDataTableValue",
            deferred(
                "Model requires InstanceId, DataTableId, Values. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates multiple data table values using all properties from BatchCreateDataTableValue. System managed values are not modifiable by customers. The operation requires proper lock versions to prevent concurrent modification…",
            ),
        ),
        (
            "ClaimPhoneNumber",
            deferred(
                "Model requires PhoneNumber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Claims an available phone number to your Connect Customer instance or traffic distribution group. You can call this API only in the same Amazon Web Services Region where the Connect Customer instance or traffic distribution group…",
            ),
        ),
        (
            "CompleteAttachedFileUpload",
            deferred(
                "Model requires InstanceId, FileId, AssociatedResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows you to confirm that the attached file has been uploaded using the pre-signed URL provided in the StartAttachedFileUpload API.",
            ),
        ),
        (
            "CreateAgentStatus",
            deferred(
                "Model requires InstanceId, Name, State. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an agent status for the specified Connect Customer instance.",
            ),
        ),
        (
            "CreateAttachedFile",
            deferred(
                "Model requires InstanceId, FileUseCaseType, FileSourceUri, AssociatedResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an attached file for a completed voice contact by copying a recording from a source S3 URI into Connect Customer managed storage. Use this API to attach voice recordings to contacts for downstream processing such as…",
            ),
        ),
        (
            "CreateAuthCode",
            deferred(
                "Model requires InstanceId, Scope, SessionInactivityDurationMinutes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an authorization code for the specified Connect Customer instance. The authorization code can be used to establish a session with scoped permissions defined by the specified scope parameters.",
            ),
        ),
        (
            "CreateContact",
            deferred(
                "Model requires InstanceId, Channel, InitiationMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Only the VOICE, EMAIL, and TASK channels are supported. For VOICE: The supported initiation method is TRANSFER . The contacts created with this initiation method have a subtype connect:ExternalAudio . For EMAIL: The supported…",
            ),
        ),
        (
            "CreateContactFlow",
            deferred(
                "Model requires InstanceId, Name, Type, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a flow for the specified Connect Customer instance. You can also create and update flows using the Connect Customer Flow language .",
            ),
        ),
        (
            "CreateContactFlowAlias",
            deferred(
                "Model requires InstanceId, ContactFlowId, ContactFlowVersion, AliasName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a named alias that points to a specific version of a flow.",
            ),
        ),
        (
            "CreateContactFlowModule",
            deferred(
                "Model requires InstanceId, Name, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a flow module for the specified Connect Customer instance.",
            ),
        ),
        (
            "CreateContactFlowModuleAlias",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId, ContactFlowModuleVersion, AliasName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a named alias that points to a specific version of a contact flow module.",
            ),
        ),
        (
            "CreateContactFlowModuleVersion",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an immutable snapshot of a contact flow module, preserving its content and settings at a specific point in time for version control and rollback capabilities.",
            ),
        ),
        (
            "CreateContactFlowVersion",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Publishes a new version of the flow provided. Versions are immutable and monotonically increasing. If the FlowContentSha256 provided is different from the FlowContentSha256 of the $LATEST published flow content, then an error is…",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateDataTable",
            deferred(
                "Model requires InstanceId, Name, TimeZone, ValueLockLevel, Status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new data table with the specified properties. Supports the creation of all table properties except for attributes and values. A table with no attributes and values is a valid state for a table. The number of tables per…",
            ),
        ),
        (
            "CreateDataTableAttribute",
            deferred(
                "Model requires InstanceId, DataTableId, Name, ValueType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds an attribute to an existing data table. Creating a new primary attribute uses the empty value for the specified value type for all existing records. This should not affect uniqueness of published data tables since the…",
            ),
        ),
        (
            "CreateEmailAddress",
            deferred(
                "Model requires InstanceId, EmailAddress. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create new email address in the specified Connect Customer instance. For more information about email addresses, see Create email addresses in the Connect Customer Administrator Guide.",
            ),
        ),
        (
            "CreateEvaluationForm",
            deferred(
                "Model requires InstanceId, Title, Items. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an evaluation form in the specified Connect Customer instance. The form can be used to define questions related to agent performance, and create sections to organize such questions. Question and section identifiers cannot…",
            ),
        ),
        (
            "CreateExtractionDefinition",
            deferred(
                "Model requires InstanceId, Name, ExtractionConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an extraction definition in the specified Connect Customer instance. An extraction definition specifies how structured data is extracted from customer interactions using generative AI, including the prompt hint that…",
            ),
        ),
        (
            "CreateHoursOfOperation",
            deferred(
                "Model requires InstanceId, Name, TimeZone, Config. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates hours of operation.",
            ),
        ),
        (
            "CreateHoursOfOperationOverride",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, Name, Config, EffectiveFrom, EffectiveTill. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an hours of operation override in an Connect Customer hours of operation resource.",
            ),
        ),
        (
            "CreateInstance",
            deferred(
                "Model requires IdentityManagementType, InboundCallsEnabled, OutboundCallsEnabled. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Initiates an Connect Customer instance with all the supported channels enabled. It does not attach any storage, such as Amazon Simple Storage Service…",
            ),
        ),
        (
            "CreateIntegrationAssociation",
            deferred(
                "Model requires InstanceId, IntegrationType, IntegrationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Web Services resource association with an Connect Customer instance.",
            ),
        ),
        (
            "CreateMetric",
            deferred(
                "Model requires InstanceId, Name, MetricCalculation, Unit. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new metric definition for the specified Connect Customer instance. You can create custom metrics that use formulas referencing existing Amazon Web Services-managed metrics, optionally with filters applied.",
            ),
        ),
        (
            "CreateNotification",
            deferred(
                "Model requires InstanceId, Recipients, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new notification to be delivered to specified recipients. Notifications can include localized content with links, and an optional expiration time. Recipients can be specified as individual user ARNs or instance ARNs to…",
            ),
        ),
        (
            "CreateParticipant",
            deferred(
                "Model requires InstanceId, ContactId, ParticipantDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a new participant into an on-going chat contact or webRTC call. For more information, see Customize chat flow experiences by integrating custom participants or Enable multi-user web, in-app, and video calling .",
            ),
        ),
        (
            "CreatePersistentContactAssociation",
            deferred(
                "Model requires InstanceId, InitialContactId, RehydrationType, SourceContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables rehydration of chats for the lifespan of a contact. For more information about chat rehydration, see Enable persistent chat in the Connect Customer Administrator Guide .",
            ),
        ),
        (
            "CreatePredefinedAttribute",
            deferred(
                "Model requires InstanceId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new predefined attribute for the specified Connect Customer instance. A predefined attribute is made up of a name and a value. For the predefined attributes per instance quota, see Connect Customer quotas . Use cases…",
            ),
        ),
        (
            "CreatePrompt",
            deferred(
                "Model requires InstanceId, Name, S3Uri. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a prompt. For more information about prompts, such as supported file types and maximum length, see Create prompts in the Connect Customer Administrator Guide .",
            ),
        ),
        (
            "CreatePushNotificationRegistration",
            deferred(
                "Model requires InstanceId, PinpointAppArn, DeviceToken, DeviceType, ContactConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates registration for a device token and a chat contact to receive real-time push notifications. For more information about push notifications, see Set up push notifications in Connect Customer for mobile chat in the Connect…",
            ),
        ),
        (
            "CreateQueue",
            deferred(
                "Model requires InstanceId, Name, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new queue for the specified Connect Customer instance. If the phone number is claimed to a traffic distribution group that was created in the same Region as the Connect Customer instance where you are calling this API,…",
            ),
        ),
        (
            "CreateQuickConnect",
            deferred(
                "Model requires InstanceId, Name, QuickConnectConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a quick connect for the specified Connect Customer instance.",
            ),
        ),
        (
            "CreateRoutingProfile",
            deferred(
                "Model requires InstanceId, Name, Description, DefaultOutboundQueueId, MediaConcurrencies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new routing profile.",
            ),
        ),
        (
            "CreateRule",
            deferred(
                "Model requires InstanceId, Name, TriggerEventSource, Function, Actions, PublishStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a rule for the specified Connect Customer instance. Use the Rules Function language to code conditions for the rule.",
            ),
        ),
        (
            "CreateSecurityProfile",
            deferred(
                "Model requires SecurityProfileName, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a security profile. For information about security profiles, see Security Profiles in the Connect Customer Administrator Guide . For a mapping of the API name and user interface name of the security profile permissions,…",
            ),
        ),
        (
            "CreateTaskTemplate",
            deferred(
                "Model requires InstanceId, Name, Fields. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new task template in the specified Connect Customer instance.",
            ),
        ),
        (
            "CreateTestCase",
            deferred(
                "Model requires InstanceId, Name, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a test case with its content and metadata for the specified Amazon Connect instance.",
            ),
        ),
        (
            "CreateTrafficDistributionGroup",
            deferred(
                "Model requires Name, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a traffic distribution group given an Connect Customer instance that has been replicated. The SignInConfig distribution is available only on a default TrafficDistributionGroup (see the IsDefault parameter in the…",
            ),
        ),
        (
            "CreateUseCase",
            deferred(
                "Model requires InstanceId, IntegrationAssociationId, UseCaseType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a use case for an integration association.",
            ),
        ),
        (
            "CreateUser",
            deferred(
                "Model requires Username, SecurityProfileIds, RoutingProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a user account for the specified Connect Customer instance. Certain UserIdentityInfo parameters are required in some situations. For example, Email , FirstName and LastName are required if you are using Connect Customer…",
            ),
        ),
        (
            "CreateUserHierarchyGroup",
            deferred(
                "Model requires Name, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new user hierarchy group.",
            ),
        ),
        (
            "CreateView",
            deferred(
                "Model requires InstanceId, Status, Content, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new view with the possible status of SAVED or PUBLISHED . The views will have a unique name for each connect instance. It performs basic content validation if the status is SAVED or full content validation if the status…",
            ),
        ),
        (
            "CreateViewVersion",
            deferred(
                "Model requires InstanceId, ViewId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Publishes a new version of the view identifier. Versions are immutable and monotonically increasing. It returns the highest version if there is no change in content compared to that version. An error is displayed if the supplied…",
            ),
        ),
        (
            "CreateVocabulary",
            deferred(
                "Model requires InstanceId, VocabularyName, LanguageCode, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom vocabulary associated with your Connect Customer instance. You can set a custom vocabulary to be your default vocabulary for a given language. Contact Lens for Connect Customer uses the default vocabulary in…",
            ),
        ),
        (
            "CreateWorkspace",
            deferred(
                "Model requires InstanceId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a workspace that defines the user experience by mapping views to pages. Workspaces can be assigned to users or routing profiles.",
            ),
        ),
        (
            "CreateWorkspacePage",
            deferred(
                "Model requires InstanceId, WorkspaceId, ResourceArn, Page. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a view with a page in a workspace, defining what users see when they navigate to that page.",
            ),
        ),
        (
            "DeactivateEvaluationForm",
            deferred(
                "Model requires InstanceId, EvaluationFormId, EvaluationFormVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deactivates an evaluation form in the specified Connect Customer instance. After a form is deactivated, it is no longer available for users to start new evaluations based on the form.",
            ),
        ),
        (
            "DeleteAttachedFile",
            deferred(
                "Model requires InstanceId, FileId, AssociatedResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an attached file along with the underlying S3 Object. The attached file is permanently deleted if S3 bucket versioning is not enabled.",
            ),
        ),
        (
            "DeleteContactData",
            deferred(
                "Model requires InstanceId, ContactId, ContactFields. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified fields containing personally identifiable information (PII) from a contact in the specified Connect Customer instance. We redact PII (such as customer endpoints, additional email recipients, and the email…",
            ),
        ),
        (
            "DeleteContactEvaluation",
            deferred(
                "Model requires InstanceId, EvaluationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a contact evaluation in the specified Connect Customer instance.",
            ),
        ),
        (
            "DeleteContactFlow",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a flow for the specified Connect Customer instance.",
            ),
        ),
        (
            "DeleteContactFlowAlias",
            deferred(
                "Model requires InstanceId, ContactFlowId, AliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an alias from a flow without deleting the version that the alias points to.",
            ),
        ),
        (
            "DeleteContactFlowModule",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified flow module.",
            ),
        ),
        (
            "DeleteContactFlowModuleAlias",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId, AliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an alias reference, breaking the named connection to the underlying module version without affecting the version itself.",
            ),
        ),
        (
            "DeleteContactFlowModuleVersion",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId, ContactFlowModuleVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a specific version of a contact flow module.",
            ),
        ),
        (
            "DeleteContactFlowVersion",
            deferred(
                "Model requires InstanceId, ContactFlowId, ContactFlowVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the particular version specified in flow version identifier.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DeleteDataTable",
            deferred(
                "Model requires InstanceId, DataTableId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a data table and all associated attributes, versions, audits, and values. Does not update any references to the data table, even from other data tables. This includes dynamic values and conditional validations. System…",
            ),
        ),
        (
            "DeleteDataTableAttribute",
            deferred(
                "Model requires InstanceId, DataTableId, AttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an attribute and all its values from a data table.",
            ),
        ),
        (
            "DeleteEmailAddress",
            deferred(
                "Model requires InstanceId, EmailAddressId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes email address from the specified Connect Customer instance.",
            ),
        ),
        (
            "DeleteEvaluationForm",
            deferred(
                "Model requires InstanceId, EvaluationFormId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an evaluation form in the specified Connect Customer instance. If the version property is provided, only the specified version of the evaluation form is deleted. If no version is provided, then the full form (all…",
            ),
        ),
        (
            "DeleteExtractionDefinition",
            deferred(
                "Model requires InstanceId, ExtractionDefinitionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an extraction definition from the specified Connect Customer instance.",
            ),
        ),
        (
            "DeleteHoursOfOperation",
            deferred(
                "Model requires InstanceId, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an hours of operation.",
            ),
        ),
        (
            "DeleteHoursOfOperationOverride",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, HoursOfOperationOverrideId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an hours of operation override in an Connect Customer hours of operation resource.",
            ),
        ),
        (
            "DeleteInstance",
            deferred(
                "Model requires InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Deletes the Connect Customer instance. For more information, see Delete your Connect Customer instance in the Connect Customer Administrator Guide .…",
            ),
        ),
        (
            "DeleteIntegrationAssociation",
            deferred(
                "Model requires InstanceId, IntegrationAssociationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Amazon Web Services resource association from an Connect Customer instance. The association must not have any use cases associated with it.",
            ),
        ),
        (
            "DeleteMetric",
            deferred(
                "Model requires InstanceId, MetricId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing metric from the specified Connect Customer instance. This operation fails with ResourceConflictException if the metric is currently in use in a dashboard.",
            ),
        ),
        (
            "DeleteNotification",
            deferred(
                "Model requires InstanceId, NotificationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a notification. Once deleted, the notification is no longer visible to all users and cannot be managed through the Admin Website or APIs.",
            ),
        ),
        (
            "DeletePredefinedAttribute",
            deferred(
                "Model requires InstanceId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a predefined attribute from the specified Connect Customer instance.",
            ),
        ),
        (
            "DeletePrompt",
            deferred(
                "Model requires InstanceId, PromptId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a prompt.",
            ),
        ),
        (
            "DeletePushNotificationRegistration",
            deferred(
                "Model requires InstanceId, RegistrationId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes registration for a device token and a chat contact.",
            ),
        ),
        (
            "DeleteQueue",
            deferred(
                "Model requires InstanceId, QueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a queue.",
            ),
        ),
        (
            "DeleteQuickConnect",
            deferred(
                "Model requires InstanceId, QuickConnectId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a quick connect. After calling DeleteUser , it's important to call DeleteQuickConnect to delete any records related to the deleted users. This will help you: Avoid dangling resources that impact your service quotas.…",
            ),
        ),
        (
            "DeleteRoutingProfile",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a routing profile.",
            ),
        ),
        (
            "DeleteRule",
            deferred(
                "Model requires InstanceId, RuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a rule for the specified Connect Customer instance.",
            ),
        ),
        (
            "DeleteSecurityProfile",
            deferred(
                "Model requires InstanceId, SecurityProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a security profile.",
            ),
        ),
        (
            "DeleteSession",
            deferred(
                "Model requires InstanceId, SessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a session for the specified Connect Customer instance.",
            ),
        ),
        (
            "DeleteTaskTemplate",
            deferred(
                "Model requires InstanceId, TaskTemplateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the task template.",
            ),
        ),
        (
            "DeleteTestCase",
            deferred(
                "Model requires InstanceId, TestCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the test case that has already been created for the specified Amazon Connect instance.",
            ),
        ),
        (
            "DeleteTrafficDistributionGroup",
            deferred(
                "Model requires TrafficDistributionGroupId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a traffic distribution group. This API can be called only in the Region where the traffic distribution group is created. For more information about deleting traffic distribution groups, see Delete traffic distribution…",
            ),
        ),
        (
            "DeleteUseCase",
            deferred(
                "Model requires InstanceId, IntegrationAssociationId, UseCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a use case from an integration association.",
            ),
        ),
        (
            "DeleteUser",
            deferred(
                "Model requires InstanceId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a user account from the specified Connect Customer instance. For information about what happens to a user's data when their account is deleted, see Delete Users from Your Connect Customer Instance in the Connect Customer…",
            ),
        ),
        (
            "DeleteUserHierarchyGroup",
            deferred(
                "Model requires HierarchyGroupId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing user hierarchy group. It must not be associated with any agents or have any active child groups.",
            ),
        ),
        (
            "DeleteView",
            deferred(
                "Model requires InstanceId, ViewId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the view entirely. It deletes the view and all associated qualifiers (versions and aliases).",
            ),
        ),
        (
            "DeleteViewVersion",
            deferred(
                "Model requires InstanceId, ViewId, ViewVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the particular version specified in ViewVersion identifier.",
            ),
        ),
        (
            "DeleteVocabulary",
            deferred(
                "Model requires InstanceId, VocabularyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the vocabulary that has the given identifier.",
            ),
        ),
        (
            "DeleteWorkspace",
            deferred(
                "Model requires InstanceId, WorkspaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a workspace and removes all associated view and resource assignments.",
            ),
        ),
        (
            "DeleteWorkspaceMedia",
            deferred(
                "Model requires InstanceId, WorkspaceId, MediaType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a media asset (such as a logo) from a workspace.",
            ),
        ),
        (
            "DeleteWorkspacePage",
            deferred(
                "Model requires InstanceId, WorkspaceId, Page. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between a view and a page in a workspace. The page will display the default view after deletion.",
            ),
        ),
        (
            "DescribeAgentStatus",
            deferred(
                "Model requires InstanceId, AgentStatusId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an agent status.",
            ),
        ),
        (
            "DescribeAttachedFilesConfiguration",
            deferred(
                "Model requires InstanceId, AttachmentScope. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the attached files configuration for the specified Connect Customer instance and attachment scope. If a custom configuration exists for the specified attachment scope, the custom configuration is returned. If no custom…",
            ),
        ),
        (
            "DescribeAuthenticationProfile",
            deferred(
                "Model requires AuthenticationProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. To request access to this API, contact Amazon Web Services Support. Describes the target authentication profile.",
            ),
        ),
        (
            "DescribeContact",
            deferred(
                "Model requires InstanceId, ContactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Describes the specified contact. Use cases Following are common uses cases for this API: Retrieve contact information such as the caller's phone number…",
            ),
        ),
        (
            "DescribeContactEvaluation",
            deferred(
                "Model requires InstanceId, EvaluationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a contact evaluation in the specified Connect Customer instance.",
            ),
        ),
        (
            "DescribeContactFlow",
            deferred(
                "Model requires InstanceId, ContactFlowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified flow. You can also create and update flows using the Connect Customer Flow language . Use the $SAVED alias in the request to describe the SAVED content of a Flow. For example,…",
            ),
        ),
        (
            "DescribeContactFlowAlias",
            deferred(
                "Model requires InstanceId, ContactFlowId, AliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specific flow alias, including the version that it points to.",
            ),
        ),
        (
            "DescribeContactFlowModule",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the specified flow module. Use the $SAVED alias in the request to describe the SAVED content of a Flow. For example, arn:aws:.../contact-flow/{id}:$SAVED . After a flow is published, $SAVED needs to be supplied to view…",
            ),
        ),
        (
            "DescribeContactFlowModuleAlias",
            deferred(
                "Model requires InstanceId, ContactFlowModuleId, AliasId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific alias, including which version it currently points to and its metadata.",
            ),
        ),
        (
            "DescribeDataTable",
            deferred(
                "Model requires InstanceId, DataTableId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all properties for a data table except for attributes and values. All properties from CreateDataTable are returned as well as properties for region replication, versioning, and system tables. \"Describe\" is a deprecated…",
            ),
        ),
    ])
}
