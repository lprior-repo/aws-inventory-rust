use crate::kind_map::{
    ApiOnlyMapping, CfnMapping, OperationMapping, RelationMapping, RelationPath, RelationSource,
    RelationTarget,
};

pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::IoTWireless::NetworkAnalyzerConfiguration"),
    CfnMapping::canonical("AWS::IoTWireless::WirelessDeviceImportTask"),
    CfnMapping::canonical("AWS::IoTWireless::TaskDefinition"),
    CfnMapping::canonical("AWS::IVS::StreamKey"),
    CfnMapping::canonical("AWS::IVS::PublicKey"),
    CfnMapping::canonical("AWS::IVS::IngestConfiguration"),
    CfnMapping::canonical("AWS::HealthLake::DataTransformationProfile"),
];

pub(super) const API_ONLY: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsAccountAssociation",
        reason: "Pinned registry has no managed-integrations account-association kind; safe summary has its own association ARN.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsDestination",
        reason: "Pinned registry has no managed-integrations notification destination kind. Name identifies this configuration; DeliveryDestinationArn identifies its referenced delivery target.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsEventLogConfiguration",
        reason: "Pinned registry has no managed-integrations event-log configuration kind; list exposes its configuration Id, not log events.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsNotificationConfiguration",
        reason: "Pinned registry has no managed-integrations notification configuration kind; EventType is its typed lookup key and DestinationName is a reference.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTaskConfiguration",
        reason: "Pinned registry has no managed-integrations OTA task configuration kind; summary identifies TaskConfigurationId without OTA content.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTask",
        reason: "Pinned registry has no managed-integrations OTA task kind; summary has TaskArn and status, not device execution payloads.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IVSComposition",
        reason: "Pinned registry has no IVS Composition kind; account-owned composition summary has its own ARN and destination lifecycle metadata, not stream credentials or media.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::HealthLakeDataTransformationProfileVersion",
        reason: "CFN DataTransformationProfile represents the profile, not every draft/published version; version summaries are source-format/profile/version qualified and contain no transformation definition.",
    },
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "iot-managed-integrations",
        operation: "ListAccountAssociations",
        kind: "AWS::ApiOnly::IoTManagedIntegrationsAccountAssociation",
    },
    OperationMapping {
        service: "iot-managed-integrations",
        operation: "ListDestinations",
        kind: "AWS::ApiOnly::IoTManagedIntegrationsDestination",
    },
    OperationMapping {
        service: "iot-managed-integrations",
        operation: "ListEventLogConfigurations",
        kind: "AWS::ApiOnly::IoTManagedIntegrationsEventLogConfiguration",
    },
    OperationMapping {
        service: "iot-managed-integrations",
        operation: "ListNotificationConfigurations",
        kind: "AWS::ApiOnly::IoTManagedIntegrationsNotificationConfiguration",
    },
    OperationMapping {
        service: "iot-managed-integrations",
        operation: "ListOtaTaskConfigurations",
        kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTaskConfiguration",
    },
    OperationMapping {
        service: "iot-managed-integrations",
        operation: "ListOtaTasks",
        kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTask",
    },
    OperationMapping {
        service: "iot-wireless",
        operation: "ListNetworkAnalyzerConfigurations",
        kind: "AWS::IoTWireless::NetworkAnalyzerConfiguration",
    },
    OperationMapping {
        service: "iot-wireless",
        operation: "ListWirelessDeviceImportTasks",
        kind: "AWS::IoTWireless::WirelessDeviceImportTask",
    },
    OperationMapping {
        service: "iot-wireless",
        operation: "ListWirelessGatewayTaskDefinitions",
        kind: "AWS::IoTWireless::TaskDefinition",
    },
    OperationMapping {
        service: "ivs",
        operation: "ListStreamKeys",
        kind: "AWS::IVS::StreamKey",
    },
    OperationMapping {
        service: "ivs-realtime",
        operation: "ListPublicKeys",
        kind: "AWS::IVS::PublicKey",
    },
    OperationMapping {
        service: "ivs-realtime",
        operation: "ListIngestConfigurations",
        kind: "AWS::IVS::IngestConfiguration",
    },
    OperationMapping {
        service: "ivs-realtime",
        operation: "ListCompositions",
        kind: "AWS::ApiOnly::IVSComposition",
    },
    OperationMapping {
        service: "healthlake",
        operation: "ListDataTransformationProfiles",
        kind: "AWS::HealthLake::DataTransformationProfile",
    },
    OperationMapping {
        service: "healthlake",
        operation: "ListDataTransformationProfileVersions",
        kind: "AWS::ApiOnly::HealthLakeDataTransformationProfileVersion",
    },
];

pub(super) const RELATIONS: &[RelationMapping<'static>] = &[
    RelationMapping {
        source_kind: "AWS::ApiOnly::IoTManagedIntegrationsNotificationConfiguration",
        source: RelationSource::SelectedOutput(RelationPath::Fields("DestinationName")),
        target_kind: "AWS::ApiOnly::IoTManagedIntegrationsDestination",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("Name")),
        relation: "DELIVERS_TO",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTask",
        source: RelationSource::SelectedOutput(RelationPath::Fields("TaskConfigurationId")),
        target_kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTaskConfiguration",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("TaskConfigurationId")),
        relation: "USES_CONFIGURATION",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::IVSComposition",
        source: RelationSource::SelectedOutput(RelationPath::Fields("stageArn")),
        target_kind: "AWS::IVS::Stage",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("arn")),
        relation: "IN_STAGE",
        scope: "account_region",
        guard: None,
    },
];

pub(super) const REQUEST_RELATIONS: &[RelationMapping<'static>] = &[
    RelationMapping {
        source_kind: "AWS::IVS::StreamKey",
        source: RelationSource::Request(RelationPath::Fields("channelArn")),
        target_kind: "AWS::IVS::Channel",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("arn")),
        relation: "IN_CHANNEL",
        scope: "account_region",
        guard: None,
    },
    RelationMapping {
        source_kind: "AWS::ApiOnly::HealthLakeDataTransformationProfileVersion",
        source: RelationSource::Request(RelationPath::Fields("ProfileId")),
        target_kind: "AWS::HealthLake::DataTransformationProfile",
        target: RelationTarget::SelectedOutput(RelationPath::Fields("ProfileId")),
        relation: "VERSION_OF",
        scope: "account_region",
        guard: None,
    },
];
