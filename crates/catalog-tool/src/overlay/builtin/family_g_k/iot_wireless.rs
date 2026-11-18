//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iot-wireless/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListWirelessDevices",
            policy::query(
                policy::enumerate(
                    "WirelessDeviceList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListWirelessGateways",
            policy::query(
                policy::enumerate(
                    "WirelessGatewayList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListDeviceProfiles",
            policy::query(
                policy::enumerate(
                    "DeviceProfileList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListServiceProfiles",
            policy::query(
                policy::enumerate(
                    "ServiceProfileList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListDestinations",
            policy::query(
                policy::enumerate(
                    "DestinationList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListMulticastGroups",
            policy::query(
                policy::enumerate(
                    "MulticastGroupList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListFuotaTasks",
            policy::query(
                policy::enumerate(
                    "FuotaTaskList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListNetworkAnalyzerConfigurations",
            policy::query(
                policy::enumerate(
                    "NetworkAnalyzerConfigurationList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn", "Name"],
            )?,
        ),
        // Explicit modeled tokens: these two APIs lack Smithy paginated traits.
        // Import Sidewalk information contains S3 file references/role/destination, not file bytes or keys.
        (
            "ListWirelessDeviceImportTasks",
            policy::query(
                policy::enumerate(
                    "WirelessDeviceImportTaskList",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn", "Id"],
            )?,
        ),
        // LoRaWAN task entries contain current/update firmware version metadata only.
        (
            "ListWirelessGatewayTaskDefinitions",
            policy::query(
                policy::enumerate(
                    "TaskDefinitions",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn", "Id"],
            )?,
        ),
    ];
    policy::finish(
        "iot-wireless",
        approved,
        DEFERRED,
        "Approved lists expose device/gateway/profile/destination/multicast/FUOTA metadata. GetWirelessDevice returns application/network session keys and is denied in full. Other credentials, payloads and child families remain unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetDestination",
    "GetDeviceProfile",
    "GetEventConfigurationByResourceTypes",
    "GetFuotaTask",
    "GetLogLevelsByResourceTypes",
    "GetMetricConfiguration",
    "GetMetrics",
    "GetMulticastGroup",
    "GetMulticastGroupSession",
    "GetNetworkAnalyzerConfiguration",
    "GetPartnerAccount",
    "GetPosition",
    "GetPositionConfiguration",
    "GetPositionEstimate",
    "GetResourceEventConfiguration",
    "GetResourceLogLevel",
    "GetResourcePosition",
    "GetServiceEndpoint",
    "GetServiceProfile",
    "GetWirelessDeviceImportTask",
    "GetWirelessDeviceStatistics",
    "GetWirelessGateway",
    "GetWirelessGatewayCertificate",
    "GetWirelessGatewayFirmwareInformation",
    "GetWirelessGatewayStatistics",
    "GetWirelessGatewayTask",
    "GetWirelessGatewayTaskDefinition",
    "ListDevicesForWirelessDeviceImportTask",
    "ListEventConfigurations",
    "ListMulticastGroupsByFuotaTask",
    "ListPartnerAccounts",
    "ListPositionConfigurations",
    "ListQueuedMessages",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AssociateAwsAccountWithPartnerAccount",
    "AssociateMulticastGroupWithFuotaTask",
    "AssociateWirelessDeviceWithFuotaTask",
    "AssociateWirelessDeviceWithMulticastGroup",
    "AssociateWirelessDeviceWithThing",
    "AssociateWirelessGatewayWithCertificate",
    "AssociateWirelessGatewayWithThing",
    "CancelMulticastGroupSession",
    "CreateDestination",
    "CreateDeviceProfile",
    "CreateFuotaTask",
    "CreateMulticastGroup",
    "CreateNetworkAnalyzerConfiguration",
    "CreateServiceProfile",
    "CreateWirelessDevice",
    "CreateWirelessGateway",
    "CreateWirelessGatewayTask",
    "CreateWirelessGatewayTaskDefinition",
    "DeleteDestination",
    "DeleteDeviceProfile",
    "DeleteFuotaTask",
    "DeleteMulticastGroup",
    "DeleteNetworkAnalyzerConfiguration",
    "DeleteQueuedMessages",
    "DeleteServiceProfile",
    "DeleteWirelessDevice",
    "DeleteWirelessDeviceImportTask",
    "DeleteWirelessGateway",
    "DeleteWirelessGatewayTask",
    "DeleteWirelessGatewayTaskDefinition",
    "DeregisterWirelessDevice",
    "DisassociateAwsAccountFromPartnerAccount",
    "DisassociateMulticastGroupFromFuotaTask",
    "DisassociateWirelessDeviceFromFuotaTask",
    "DisassociateWirelessDeviceFromMulticastGroup",
    "DisassociateWirelessDeviceFromThing",
    "DisassociateWirelessGatewayFromCertificate",
    "DisassociateWirelessGatewayFromThing",
    "PutPositionConfiguration",
    "PutResourceLogLevel",
    "ResetAllResourceLogLevels",
    "ResetResourceLogLevel",
    "SendDataToMulticastGroup",
    "SendDataToWirelessDevice",
    "StartBulkAssociateWirelessDeviceWithMulticastGroup",
    "StartBulkDisassociateWirelessDeviceFromMulticastGroup",
    "StartFuotaTask",
    "StartMulticastGroupSession",
    "StartSingleWirelessDeviceImportTask",
    "StartWirelessDeviceImportTask",
    "TagResource",
    "TestWirelessDevice",
    "UntagResource",
    "UpdateDestination",
    "UpdateEventConfigurationByResourceTypes",
    "UpdateFuotaTask",
    "UpdateLogLevelsByResourceTypes",
    "UpdateMetricConfiguration",
    "UpdateMulticastGroup",
    "UpdateNetworkAnalyzerConfiguration",
    "UpdatePartnerAccount",
    "UpdatePosition",
    "UpdateResourceEventConfiguration",
    "UpdateResourcePosition",
    "UpdateWirelessDevice",
    "UpdateWirelessDeviceImportTask",
    "UpdateWirelessGateway",
];

const SECRET: &[&str] = &["GetWirelessDevice"];
