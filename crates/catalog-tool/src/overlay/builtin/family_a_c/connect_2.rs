// Pinned source: .local/aws-models/models/connect/service/2017-08-08/connect-2017-08-08.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateDataTableMetadata",
            deferred(
                "Model requires InstanceId, DataTableId, Name, ValueLockLevel, TimeZone. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the metadata properties of a data table. Accepts all fields similar to CreateDataTable, except for fields and tags. There are no other granular update endpoints. It does not act as a patch operation - all properties must…",
            ),
        ),
        (
            "UpdateDataTablePrimaryValues",
            deferred(
                "Model requires InstanceId, DataTableId, PrimaryValues, NewPrimaryValues, LockVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the primary values for a record. This operation affects all existing values that are currently associated to the record and its primary values. Users that have restrictions on attributes and/or primary values are not…",
            ),
        ),
        (
            "UpdateEmailAddressMetadata",
            deferred(
                "Model requires InstanceId, EmailAddressId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an email address metadata. For more information about email addresses, see Create email addresses in the Connect Customer Administrator Guide.",
            ),
        ),
        (
            "UpdateEvaluationForm",
            deferred(
                "Model requires InstanceId, EvaluationFormId, EvaluationFormVersion, Title, Items. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates details about a specific evaluation form version in the specified Connect Customer instance. Question and section identifiers cannot be duplicated within the same evaluation form. This operation does not support partial…",
            ),
        ),
        (
            "UpdateExtractionDefinition",
            deferred(
                "Model requires ExtractionDefinitionId, InstanceId, Name, ExtractionConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an extraction definition in the specified Connect Customer instance.",
            ),
        ),
        (
            "UpdateHoursOfOperation",
            deferred(
                "Model requires InstanceId, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the hours of operation.",
            ),
        ),
        (
            "UpdateHoursOfOperationOverride",
            deferred(
                "Model requires InstanceId, HoursOfOperationId, HoursOfOperationOverrideId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update the hours of operation override.",
            ),
        ),
        (
            "UpdateInstanceAttribute",
            deferred(
                "Model requires InstanceId, AttributeType, Value. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Updates the value for the specified attribute type.",
            ),
        ),
        (
            "UpdateInstanceStorageConfig",
            deferred(
                "Model requires InstanceId, AssociationId, ResourceType, StorageConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API is in preview release for Connect Customer and is subject to change. Updates an existing configuration for a resource type. This API is idempotent.",
            ),
        ),
        (
            "UpdateMetricContent",
            deferred(
                "Model requires InstanceId, MetricId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the calculation, unit, and/or trend indicator of an existing metric in the specified Connect Customer instance.",
            ),
        ),
        (
            "UpdateMetricMetadata",
            deferred(
                "Model requires InstanceId, MetricId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name and/or description of an existing metric in the specified Connect Customer instance.",
            ),
        ),
        (
            "UpdateNotificationContent",
            deferred(
                "Model requires InstanceId, NotificationId, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the localized content of an existing notification. This operation applies to all users for whom the notification was sent.",
            ),
        ),
        (
            "UpdateOutboundEmailContactMessage",
            deferred(
                "Model requires InstanceId, ContactId, EmailMessage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the draft outbound email message for a contact. Use this operation to save changes to an in-progress outbound email reply before you send it. This operation applies only to contacts with an OUTBOUND or AGENT_REPLY…",
            ),
        ),
        (
            "UpdateParticipantAuthentication",
            deferred(
                "Model requires State, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Instructs Connect Customer to resume the authentication process. The subsequent actions depend on the request body contents: If a code is provided : Connect retrieves the identity information from Amazon Cognito and imports it…",
            ),
        ),
        (
            "UpdateParticipantRoleConfig",
            deferred(
                "Model requires InstanceId, ContactId, ChannelConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates timeouts for when human chat participants are to be considered idle, and when agents are automatically disconnected from a chat due to idleness. You can set four timers: Customer idle timeout Customer auto-disconnect…",
            ),
        ),
        (
            "UpdatePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates your claimed phone number from its current Connect Customer instance or traffic distribution group to another Connect Customer instance or traffic distribution group in the same Amazon Web Services Region. After using…",
            ),
        ),
        (
            "UpdatePhoneNumberMetadata",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a phone number’s metadata. To verify the status of a previous UpdatePhoneNumberMetadata operation, call the DescribePhoneNumber API.",
            ),
        ),
        (
            "UpdatePredefinedAttribute",
            deferred(
                "Model requires InstanceId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a predefined attribute for the specified Connect Customer instance. A predefined attribute is made up of a name and a value. For the predefined attributes per instance quota, see Connect Customer quotas . Use cases…",
            ),
        ),
        (
            "UpdatePrompt",
            deferred(
                "Model requires InstanceId, PromptId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a prompt.",
            ),
        ),
        (
            "UpdateQueueHoursOfOperation",
            deferred(
                "Model requires InstanceId, QueueId, HoursOfOperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the hours of operation for the specified queue.",
            ),
        ),
        (
            "UpdateQueueMaxContacts",
            deferred(
                "Model requires InstanceId, QueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the maximum number of contacts allowed in a queue before it is considered full.",
            ),
        ),
        (
            "UpdateQueueName",
            deferred(
                "Model requires InstanceId, QueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name and description of a queue. At least Name or Description must be provided.",
            ),
        ),
        (
            "UpdateQueueOutboundCallerConfig",
            deferred(
                "Model requires InstanceId, QueueId, OutboundCallerConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the outbound caller ID name, number, and outbound whisper flow for a specified queue. If the phone number is claimed to a traffic distribution group that was created in the same Region as the Connect Customer instance…",
            ),
        ),
        (
            "UpdateQueueOutboundEmailConfig",
            deferred(
                "Model requires InstanceId, QueueId, OutboundEmailConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the outbound email address Id for a specified queue.",
            ),
        ),
        (
            "UpdateQueueStatus",
            deferred(
                "Model requires InstanceId, QueueId, Status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of the queue.",
            ),
        ),
        (
            "UpdateQuickConnectConfig",
            deferred(
                "Model requires InstanceId, QuickConnectId, QuickConnectConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration settings for the specified quick connect.",
            ),
        ),
        (
            "UpdateQuickConnectName",
            deferred(
                "Model requires InstanceId, QuickConnectId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name and description of a quick connect. The request accepts the following data in JSON format. At least Name or Description must be provided.",
            ),
        ),
        (
            "UpdateRoutingProfileAgentAvailabilityTimer",
            deferred(
                "Model requires InstanceId, RoutingProfileId, AgentAvailabilityTimer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Whether agents with this routing profile will have their routing order calculated based on time since their last inbound contact or longest idle time .",
            ),
        ),
        (
            "UpdateRoutingProfileConcurrency",
            deferred(
                "Model requires InstanceId, RoutingProfileId, MediaConcurrencies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the channels that agents can handle in the Contact Control Panel (CCP) for a routing profile.",
            ),
        ),
        (
            "UpdateRoutingProfileDefaultOutboundQueue",
            deferred(
                "Model requires InstanceId, RoutingProfileId, DefaultOutboundQueueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the default outbound queue of a routing profile.",
            ),
        ),
        (
            "UpdateRoutingProfileName",
            deferred(
                "Model requires InstanceId, RoutingProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name and description of a routing profile. The request accepts the following data in JSON format. At least Name or Description must be provided.",
            ),
        ),
        (
            "UpdateRoutingProfileQueues",
            deferred(
                "Model requires InstanceId, RoutingProfileId, QueueConfigs. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the properties associated with a set of queues for a routing profile.",
            ),
        ),
        (
            "UpdateRule",
            deferred(
                "Model requires RuleId, InstanceId, Name, Function, Actions, PublishStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a rule for the specified Connect Customer instance. Use the Rules Function language to code conditions for the rule.",
            ),
        ),
        (
            "UpdateSecurityProfile",
            deferred(
                "Model requires SecurityProfileId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a security profile. For information about security profiles, see Security Profiles in the Connect Customer Administrator Guide . For a mapping of the API name and user interface name of the security profile permissions,…",
            ),
        ),
        (
            "UpdateTaskTemplate",
            deferred(
                "Model requires TaskTemplateId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates details about a specific task template in the specified Connect Customer instance. This operation does not support partial updates. Instead it does a full update of template content.",
            ),
        ),
        (
            "UpdateTestCase",
            deferred(
                "Model requires InstanceId, TestCaseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates any of the metadata for a test case, such as the name, description, and status or content of an existing test case. This API doesn't allow customers to update the tags of the test case resource for the specified Amazon…",
            ),
        ),
        (
            "UpdateTrafficDistribution",
            deferred(
                "Model requires Id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the traffic distribution for a given traffic distribution group. When you shift telephony traffic, also shift agents and/or agent sign-ins to ensure they can handle the calls in the other Region. If you don't shift the…",
            ),
        ),
        (
            "UpdateUserConfig",
            deferred(
                "Model requires UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration settings for the specified user, including per-channel auto-accept and after contact work (ACW) timeout settings. This operation replaces the UpdateUserPhoneConfig API. While UpdateUserPhoneConfig…",
            ),
        ),
        (
            "UpdateUserHierarchy",
            deferred(
                "Model requires UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns the specified hierarchy group to the specified user.",
            ),
        ),
        (
            "UpdateUserHierarchyGroupName",
            deferred(
                "Model requires Name, HierarchyGroupId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name of the user hierarchy group.",
            ),
        ),
        (
            "UpdateUserHierarchyStructure",
            deferred(
                "Model requires HierarchyStructure, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the user hierarchy structure: add, remove, and rename user hierarchy levels.",
            ),
        ),
        (
            "UpdateUserIdentityInfo",
            deferred(
                "Model requires IdentityInfo, UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the identity information for the specified user. We strongly recommend limiting who has the ability to invoke UpdateUserIdentityInfo . Someone with that ability can change the login credentials of other users by changing…",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateUserNotificationStatus",
            deferred(
                "Model requires InstanceId, NotificationId, UserId, Status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of a notification for a specific user, such as marking it as read or hidden. Users can only update notification status for notifications that have been sent to them. READ status deprioritizes the notification…",
            ),
        ),
        (
            "UpdateUserPhoneConfig",
            deferred(
                "Model requires PhoneConfig, UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the phone configuration settings for the specified user. We recommend using the UpdateUserConfig API, which supports additional functionality that is not available in the UpdateUserPhoneConfig API, such as voice…",
            ),
        ),
        (
            "UpdateUserProficiencies",
            deferred(
                "Model requires InstanceId, UserId, UserProficiencies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the properties associated with the proficiencies of a user.",
            ),
        ),
        (
            "UpdateUserRoutingProfile",
            deferred(
                "Model requires RoutingProfileId, UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns the specified routing profile to the specified user.",
            ),
        ),
        (
            "UpdateUserSecurityProfiles",
            deferred(
                "Model requires SecurityProfileIds, UserId, InstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns the specified security profiles to the specified user.",
            ),
        ),
        (
            "UpdateViewContent",
            deferred(
                "Model requires InstanceId, ViewId, Status, Content. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the view content of the given view identifier in the specified Connect Customer instance. It performs content validation if Status is set to SAVED and performs full content validation if Status is PUBLISHED . Note that…",
            ),
        ),
        (
            "UpdateViewMetadata",
            deferred(
                "Model requires InstanceId, ViewId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the view metadata. Note that either Name or Description must be provided.",
            ),
        ),
        (
            "UpdateWorkspaceMetadata",
            deferred(
                "Model requires InstanceId, WorkspaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the metadata of a workspace, such as its name and description.",
            ),
        ),
        (
            "UpdateWorkspacePage",
            deferred(
                "Model requires InstanceId, WorkspaceId, Page. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration of a page in a workspace, including the associated view and input data.",
            ),
        ),
        (
            "UpdateWorkspaceTheme",
            deferred(
                "Model requires InstanceId, WorkspaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the theme configuration for a workspace, including colors and styling.",
            ),
        ),
        (
            "UpdateWorkspaceVisibility",
            deferred(
                "Model requires InstanceId, WorkspaceId, Visibility. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the visibility setting of a workspace, controlling whether it is available to all users, assigned users only, or none.",
            ),
        ),
    ])
}
