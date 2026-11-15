//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iot-managed-integrations/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListManagedThings", policy::query(policy::enumerate("Items", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, &["Arn"])?),
        ("ListCredentialLockers", policy::query(policy::enumerate("Items", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, &["Arn"])?),
        ("ListCloudConnectors", policy::query(policy::enumerate("Items", policy::key("Id")?, Some(("NextToken", "NextToken")))?, &["Id"])?),
        ("ListConnectorDestinations", policy::query(policy::enumerate("ConnectorDestinationList", policy::key("Id")?, Some(("NextToken", "NextToken")))?, &["Id"])?),
        ("ListProvisioningProfiles", policy::query(policy::enumerate("Items", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, &["Arn"])?),
        // Whole summaries contain configuration metadata/references, never their detail payloads.
        ("ListAccountAssociations", policy::query(policy::enumerate("Items", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, &["Arn", "AccountAssociationId"])?),
        ("ListDestinations", policy::query(policy::enumerate("DestinationList", policy::key("Name")?, Some(("NextToken", "NextToken")))?, &["Name", "DeliveryDestinationArn"])?),
        ("ListEventLogConfigurations", policy::query(policy::enumerate("EventLogConfigurationList", policy::key("Id")?, Some(("NextToken", "NextToken")))?, &["Id"])?),
        ("ListNotificationConfigurations", policy::query(policy::enumerate("NotificationConfigurationList", policy::key("EventType")?, Some(("NextToken", "NextToken")))?, &["EventType", "DestinationName"])?),
        ("ListOtaTaskConfigurations", policy::query(policy::enumerate("Items", policy::key("TaskConfigurationId")?, Some(("NextToken", "NextToken")))?, &["TaskConfigurationId"])?),
        ("ListOtaTasks", policy::query(policy::enumerate("Tasks", policy::arn("TaskArn")?, Some(("NextToken", "NextToken")))?, &["TaskArn", "TaskConfigurationId"])?),
    ];
    policy::finish(
        "iot-managed-integrations", approved, DEFERRED,
        "Approved lists return configuration/association metadata and references, not credential values, device state, OTA bytes or commands. OTA configuration Name is a sensitive-annotated resource label, not a secret. Device discovery/state/schema and detail payloads remain unapproved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetAccountAssociation", "GetCloudConnector", "GetConnectorDestination", "GetCredentialLocker",
    "GetCustomEndpoint", "GetDefaultEncryptionConfiguration", "GetDestination", "GetDeviceDiscovery",
    "GetEventLogConfiguration", "GetHubConfiguration", "GetManagedThing", "GetManagedThingCapabilities",
    "GetManagedThingCertificate", "GetManagedThingConnectivityData", "GetManagedThingMetaData",
    "GetManagedThingState", "GetNotificationConfiguration", "GetOtaTask", "GetOtaTaskConfiguration",
    "GetProvisioningProfile", "GetRuntimeLogConfiguration", "GetSchemaVersion",
    "ListDeviceDiscoveries", "ListDiscoveredDevices",
    "ListManagedThingAccountAssociations", "ListManagedThingSchemas",
    "ListOtaTaskExecutions", "ListSchemaVersions",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateAccountAssociation", "CreateCloudConnector", "CreateConnectorDestination",
    "CreateCredentialLocker", "CreateDestination", "CreateEventLogConfiguration", "CreateManagedThing",
    "CreateNotificationConfiguration", "CreateOtaTask", "CreateOtaTaskConfiguration",
    "CreateProvisioningProfile", "DeleteAccountAssociation", "DeleteCloudConnector",
    "DeleteConnectorDestination", "DeleteCredentialLocker", "DeleteDestination",
    "DeleteEventLogConfiguration", "DeleteManagedThing", "DeleteNotificationConfiguration", "DeleteOtaTask",
    "DeleteOtaTaskConfiguration", "DeleteProvisioningProfile", "DeregisterAccountAssociation",
    "PutDefaultEncryptionConfiguration", "PutHubConfiguration", "PutRuntimeLogConfiguration",
    "RegisterAccountAssociation", "RegisterCustomEndpoint", "ResetRuntimeLogConfiguration",
    "SendConnectorEvent", "SendManagedThingCommand", "StartAccountAssociationRefresh",
    "StartDeviceDiscovery", "TagResource", "UntagResource", "UpdateAccountAssociation",
    "UpdateCloudConnector", "UpdateConnectorDestination", "UpdateDestination", "UpdateEventLogConfiguration",
    "UpdateManagedThing", "UpdateNotificationConfiguration", "UpdateOtaTask",
];
