use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "elementalinference", operation: "ListFeeds", kind: "AWS::ApiOnly::ElementalInferenceFeed" },
    OperationMapping { service: "emr-containers", operation: "ListVirtualClusters", kind: "AWS::EMRContainers::VirtualCluster" },
    OperationMapping { service: "emr-containers", operation: "ListSecurityConfigurations", kind: "AWS::ApiOnly::EMRContainersSecurityConfiguration" },
    OperationMapping { service: "emr-serverless", operation: "ListApplications", kind: "AWS::EMRServerless::Application" },
    OperationMapping { service: "emr", operation: "ListClusters", kind: "AWS::EMR::Cluster" },
    OperationMapping { service: "emr", operation: "ListSecurityConfigurations", kind: "AWS::EMR::SecurityConfiguration" },
    OperationMapping { service: "emr", operation: "ListStudios", kind: "AWS::EMR::Studio" },
    OperationMapping { service: "endusermessaging", operation: "ListBrandProfiles", kind: "AWS::ApiOnly::EndUserMessagingBrandProfile" },
    OperationMapping { service: "entityresolution", operation: "ListIdMappingWorkflows", kind: "AWS::EntityResolution::IdMappingWorkflow" },
    OperationMapping { service: "entityresolution", operation: "ListIdNamespaces", kind: "AWS::EntityResolution::IdNamespace" },
    OperationMapping { service: "entityresolution", operation: "ListMatchingWorkflows", kind: "AWS::EntityResolution::MatchingWorkflow" },
    OperationMapping { service: "entityresolution", operation: "ListSchemaMappings", kind: "AWS::EntityResolution::SchemaMapping" },
    OperationMapping { service: "eventbridge", operation: "ListApiDestinations", kind: "AWS::Events::ApiDestination" },
    OperationMapping { service: "eventbridge", operation: "ListArchives", kind: "AWS::Events::Archive" },
    OperationMapping { service: "eventbridge", operation: "ListConnections", kind: "AWS::Events::Connection" },
    OperationMapping { service: "eventbridge", operation: "ListEndpoints", kind: "AWS::Events::Endpoint" },
    OperationMapping { service: "eventbridge", operation: "ListEventBuses", kind: "AWS::Events::EventBus" },
    OperationMapping { service: "eventbridge", operation: "ListEventSources", kind: "AWS::ApiOnly::EventsEventSource" },
    OperationMapping { service: "eventbridge", operation: "ListReplays", kind: "AWS::ApiOnly::EventsReplay" },
    OperationMapping { service: "eventbridge", operation: "ListRules", kind: "AWS::Events::Rule" },
];
