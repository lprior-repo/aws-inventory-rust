use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "config-service", operation: "DescribeConfigRules", kind: "AWS::Config::ConfigRule" },
    OperationMapping { service: "config-service", operation: "DescribeConfigurationAggregators", kind: "AWS::Config::ConfigurationAggregator" },
    OperationMapping { service: "config-service", operation: "DescribeConfigurationRecorders", kind: "AWS::Config::ConfigurationRecorder" },
    OperationMapping { service: "config-service", operation: "DescribeConformancePacks", kind: "AWS::Config::ConformancePack" },
    OperationMapping { service: "config-service", operation: "DescribeDeliveryChannels", kind: "AWS::Config::DeliveryChannel" },
    OperationMapping { service: "config-service", operation: "DescribeOrganizationConfigRules", kind: "AWS::Config::OrganizationConfigRule" },
    OperationMapping { service: "config-service", operation: "DescribeOrganizationConformancePacks", kind: "AWS::Config::OrganizationConformancePack" },
    OperationMapping { service: "config-service", operation: "DescribeRetentionConfigurations", kind: "AWS::ApiOnly::ConfigRetentionConfiguration" },
    OperationMapping { service: "config-service", operation: "ListConnectors", kind: "AWS::ApiOnly::ConfigConnector" },
    OperationMapping { service: "config-service", operation: "ListResourceEvaluations", kind: "AWS::ApiOnly::ConfigResourceEvaluation" },
    OperationMapping { service: "config-service", operation: "ListStoredQueries", kind: "AWS::Config::StoredQuery" },
    OperationMapping { service: "connect", operation: "DescribeInstance", kind: "AWS::Connect::Instance" },
    OperationMapping { service: "connect", operation: "ListContactFlowModules", kind: "AWS::Connect::ContactFlowModule" },
    OperationMapping { service: "connect", operation: "ListContactFlows", kind: "AWS::Connect::ContactFlow" },
    OperationMapping { service: "connect", operation: "ListHoursOfOperations", kind: "AWS::Connect::HoursOfOperation" },
    OperationMapping { service: "connect", operation: "ListInstances", kind: "AWS::Connect::Instance" },
    OperationMapping { service: "connect", operation: "ListPhoneNumbersV2", kind: "AWS::Connect::PhoneNumber" },
    OperationMapping { service: "connect", operation: "ListPrompts", kind: "AWS::Connect::Prompt" },
    OperationMapping { service: "connect", operation: "ListQueues", kind: "AWS::Connect::Queue" },
    OperationMapping { service: "connect", operation: "ListRoutingProfiles", kind: "AWS::Connect::RoutingProfile" },
];
