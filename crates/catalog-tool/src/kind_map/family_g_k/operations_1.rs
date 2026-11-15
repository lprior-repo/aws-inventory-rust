use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "greengrass", operation: "ListDeviceDefinitions", kind: "AWS::Greengrass::DeviceDefinition" },
    OperationMapping { service: "greengrass", operation: "ListFunctionDefinitions", kind: "AWS::Greengrass::FunctionDefinition" },
    OperationMapping { service: "greengrass", operation: "ListResourceDefinitions", kind: "AWS::Greengrass::ResourceDefinition" },
    OperationMapping { service: "greengrass", operation: "ListSubscriptionDefinitions", kind: "AWS::Greengrass::SubscriptionDefinition" },
    OperationMapping { service: "greengrass", operation: "ListLoggerDefinitions", kind: "AWS::Greengrass::LoggerDefinition" },
    OperationMapping { service: "greengrass", operation: "ListConnectorDefinitions", kind: "AWS::Greengrass::ConnectorDefinition" },
    OperationMapping { service: "greengrassv2", operation: "ListComponents", kind: "AWS::ApiOnly::GreengrassComponent" },
    OperationMapping { service: "greengrassv2", operation: "ListComponentVersions", kind: "AWS::GreengrassV2::ComponentVersion" },
    OperationMapping { service: "greengrassv2", operation: "ListCoreDevices", kind: "AWS::ApiOnly::GreengrassCoreDevice" },
    OperationMapping { service: "greengrassv2", operation: "ListDeployments", kind: "AWS::GreengrassV2::Deployment" },
    OperationMapping { service: "greengrassv2", operation: "DescribeComponent", kind: "AWS::GreengrassV2::ComponentVersion" },
    OperationMapping { service: "groundstation", operation: "ListConfigs", kind: "AWS::GroundStation::Config" },
    OperationMapping { service: "groundstation", operation: "ListDataflowEndpointGroups", kind: "AWS::GroundStation::DataflowEndpointGroup" },
    OperationMapping { service: "groundstation", operation: "ListMissionProfiles", kind: "AWS::GroundStation::MissionProfile" },
    OperationMapping { service: "groundstation", operation: "ListSatellites", kind: "AWS::ApiOnly::GroundStationSatellite" },
    OperationMapping { service: "groundstation", operation: "GetMissionProfile", kind: "AWS::GroundStation::MissionProfile" },
    OperationMapping { service: "guardduty", operation: "ListDetectors", kind: "AWS::GuardDuty::Detector" },
    OperationMapping { service: "guardduty", operation: "ListMalwareProtectionPlans", kind: "AWS::GuardDuty::MalwareProtectionPlan" },
    OperationMapping { service: "healthlake", operation: "ListFHIRDatastores", kind: "AWS::HealthLake::FHIRDatastore" },
    OperationMapping { service: "healthlake", operation: "DescribeFHIRDatastore", kind: "AWS::HealthLake::FHIRDatastore" },
];
