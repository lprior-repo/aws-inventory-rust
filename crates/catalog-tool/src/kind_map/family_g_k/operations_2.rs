use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "identitystore", operation: "ListIdentityStores", kind: "AWS::ApiOnly::IdentityStore" },
    OperationMapping { service: "identitystore", operation: "DescribeIdentityStore", kind: "AWS::ApiOnly::IdentityStore" },
    OperationMapping { service: "imagebuilder", operation: "ListImagePipelines", kind: "AWS::ImageBuilder::ImagePipeline" },
    OperationMapping { service: "imagebuilder", operation: "ListContainerRecipes", kind: "AWS::ImageBuilder::ContainerRecipe" },
    OperationMapping { service: "imagebuilder", operation: "ListImageRecipes", kind: "AWS::ImageBuilder::ImageRecipe" },
    OperationMapping { service: "imagebuilder", operation: "ListDistributionConfigurations", kind: "AWS::ImageBuilder::DistributionConfiguration" },
    OperationMapping { service: "imagebuilder", operation: "ListInfrastructureConfigurations", kind: "AWS::ImageBuilder::InfrastructureConfiguration" },
    OperationMapping { service: "imagebuilder", operation: "ListLifecyclePolicies", kind: "AWS::ImageBuilder::LifecyclePolicy" },
    OperationMapping { service: "inspector", operation: "ListAssessmentTargets", kind: "AWS::Inspector::AssessmentTarget" },
    OperationMapping { service: "inspector", operation: "ListAssessmentTemplates", kind: "AWS::Inspector::AssessmentTemplate" },
    OperationMapping { service: "inspector2", operation: "ListFilters", kind: "AWS::InspectorV2::Filter" },
    OperationMapping { service: "inspector2", operation: "ListCisScanConfigurations", kind: "AWS::InspectorV2::CisScanConfiguration" },
    OperationMapping { service: "inspector2", operation: "ListCodeSecurityScanConfigurations", kind: "AWS::InspectorV2::CodeSecurityScanConfiguration" },
    OperationMapping { service: "inspector2", operation: "ListCodeSecurityIntegrations", kind: "AWS::InspectorV2::CodeSecurityIntegration" },
    OperationMapping { service: "interconnect", operation: "ListConnections", kind: "AWS::Interconnect::Connection" },
    OperationMapping { service: "internetmonitor", operation: "ListMonitors", kind: "AWS::InternetMonitor::Monitor" },
    OperationMapping { service: "internetmonitor", operation: "GetMonitor", kind: "AWS::InternetMonitor::Monitor" },
    OperationMapping { service: "invoicing", operation: "ListInvoiceUnits", kind: "AWS::Invoicing::InvoiceUnit" },
    OperationMapping { service: "iot", operation: "ListThingGroups", kind: "AWS::IoT::ThingGroup" },
    OperationMapping { service: "iot", operation: "ListThingTypes", kind: "AWS::IoT::ThingType" },
];
