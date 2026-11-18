use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "appfabric",
        operation: "ListIngestions",
        kind: "AWS::AppFabric::Ingestion",
    },
    OperationMapping {
        service: "appflow",
        operation: "DescribeConnectorProfiles",
        kind: "AWS::AppFlow::ConnectorProfile",
    },
    OperationMapping {
        service: "appflow",
        operation: "DescribeFlow",
        kind: "AWS::AppFlow::Flow",
    },
    OperationMapping {
        service: "appflow",
        operation: "ListConnectors",
        kind: "AWS::AppFlow::Connector",
    },
    OperationMapping {
        service: "appflow",
        operation: "ListFlows",
        kind: "AWS::AppFlow::Flow",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "GetApplication",
        kind: "AWS::AppIntegrations::Application",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "GetEventIntegration",
        kind: "AWS::AppIntegrations::EventIntegration",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "ListApplicationAssociations",
        kind: "AWS::ApiOnly::AppIntegrationsApplicationAssociation",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "ListApplications",
        kind: "AWS::AppIntegrations::Application",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "ListDataIntegrationAssociations",
        kind: "AWS::ApiOnly::AppIntegrationsDataIntegrationAssociation",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "ListDataIntegrations",
        kind: "AWS::AppIntegrations::DataIntegration",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "ListEventIntegrationAssociations",
        kind: "AWS::ApiOnly::AppIntegrationsEventIntegrationAssociation",
    },
    OperationMapping {
        service: "appintegrations",
        operation: "ListEventIntegrations",
        kind: "AWS::AppIntegrations::EventIntegration",
    },
    OperationMapping {
        service: "application-discovery-service",
        operation: "DescribeAgents",
        kind: "AWS::ApiOnly::DiscoveryAgent",
    },
    OperationMapping {
        service: "application-discovery-service",
        operation: "DescribeContinuousExports",
        kind: "AWS::ApiOnly::DiscoveryContinuousExport",
    },
    OperationMapping {
        service: "application-insights",
        operation: "DescribeApplication",
        kind: "AWS::ApplicationInsights::Application",
    },
    OperationMapping {
        service: "application-insights",
        operation: "ListApplications",
        kind: "AWS::ApplicationInsights::Application",
    },
    OperationMapping {
        service: "application-signals",
        operation: "GetServiceLevelObjective",
        kind: "AWS::ApplicationSignals::ServiceLevelObjective",
    },
    OperationMapping {
        service: "application-signals",
        operation: "ListServiceLevelObjectives",
        kind: "AWS::ApplicationSignals::ServiceLevelObjective",
    },
    OperationMapping {
        service: "applicationcostprofiler",
        operation: "GetReportDefinition",
        kind: "AWS::ApiOnly::ApplicationCostProfilerReportDefinition",
    },
];
