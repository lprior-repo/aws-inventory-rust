use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "app-mesh",
        operation: "DescribeMesh",
        kind: "AWS::AppMesh::Mesh",
    },
    OperationMapping {
        service: "app-mesh",
        operation: "ListMeshes",
        kind: "AWS::AppMesh::Mesh",
    },
    OperationMapping {
        service: "app-mesh",
        operation: "ListVirtualGateways",
        kind: "AWS::AppMesh::VirtualGateway",
    },
    OperationMapping {
        service: "app-mesh",
        operation: "ListVirtualNodes",
        kind: "AWS::AppMesh::VirtualNode",
    },
    OperationMapping {
        service: "app-mesh",
        operation: "ListVirtualRouters",
        kind: "AWS::AppMesh::VirtualRouter",
    },
    OperationMapping {
        service: "app-mesh",
        operation: "ListVirtualServices",
        kind: "AWS::AppMesh::VirtualService",
    },
    OperationMapping {
        service: "appconfig",
        operation: "GetApplication",
        kind: "AWS::AppConfig::Application",
    },
    OperationMapping {
        service: "appconfig",
        operation: "GetDeploymentStrategy",
        kind: "AWS::AppConfig::DeploymentStrategy",
    },
    OperationMapping {
        service: "appconfig",
        operation: "GetExtension",
        kind: "AWS::AppConfig::Extension",
    },
    OperationMapping {
        service: "appconfig",
        operation: "GetExtensionAssociation",
        kind: "AWS::AppConfig::ExtensionAssociation",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListApplications",
        kind: "AWS::AppConfig::Application",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListConfigurationProfiles",
        kind: "AWS::AppConfig::ConfigurationProfile",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListDeploymentStrategies",
        kind: "AWS::AppConfig::DeploymentStrategy",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListEnvironments",
        kind: "AWS::AppConfig::Environment",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListExperimentDefinitions",
        kind: "AWS::ApiOnly::AppConfigExperiment",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListExtensionAssociations",
        kind: "AWS::AppConfig::ExtensionAssociation",
    },
    OperationMapping {
        service: "appconfig",
        operation: "ListExtensions",
        kind: "AWS::AppConfig::Extension",
    },
    OperationMapping {
        service: "appfabric",
        operation: "GetAppBundle",
        kind: "AWS::AppFabric::AppBundle",
    },
    OperationMapping {
        service: "appfabric",
        operation: "ListAppAuthorizations",
        kind: "AWS::AppFabric::AppAuthorization",
    },
    OperationMapping {
        service: "appfabric",
        operation: "ListAppBundles",
        kind: "AWS::AppFabric::AppBundle",
    },
];
