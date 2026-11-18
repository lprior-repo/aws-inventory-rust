use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "datazone",
        operation: "ListDomains",
        kind: "AWS::DataZone::Domain",
    },
    OperationMapping {
        service: "datazone",
        operation: "ListProjects",
        kind: "AWS::DataZone::Project",
    },
    OperationMapping {
        service: "datazone",
        operation: "ListEnvironments",
        kind: "AWS::DataZone::Environment",
    },
    OperationMapping {
        service: "datazone",
        operation: "ListEnvironmentProfiles",
        kind: "AWS::DataZone::EnvironmentProfile",
    },
    OperationMapping {
        service: "datazone",
        operation: "ListDataSources",
        kind: "AWS::DataZone::DataSource",
    },
    OperationMapping {
        service: "dax",
        operation: "DescribeClusters",
        kind: "AWS::DAX::Cluster",
    },
    OperationMapping {
        service: "dax",
        operation: "DescribeParameterGroups",
        kind: "AWS::DAX::ParameterGroup",
    },
    OperationMapping {
        service: "dax",
        operation: "DescribeSubnetGroups",
        kind: "AWS::DAX::SubnetGroup",
    },
    OperationMapping {
        service: "deadline",
        operation: "ListFarms",
        kind: "AWS::Deadline::Farm",
    },
    OperationMapping {
        service: "deadline",
        operation: "ListLicenseEndpoints",
        kind: "AWS::Deadline::LicenseEndpoint",
    },
    OperationMapping {
        service: "deadline",
        operation: "ListMonitors",
        kind: "AWS::Deadline::Monitor",
    },
    OperationMapping {
        service: "deadline",
        operation: "ListFleets",
        kind: "AWS::Deadline::Fleet",
    },
    OperationMapping {
        service: "deadline",
        operation: "ListQueues",
        kind: "AWS::Deadline::Queue",
    },
    OperationMapping {
        service: "detective",
        operation: "ListGraphs",
        kind: "AWS::Detective::Graph",
    },
    OperationMapping {
        service: "device-farm",
        operation: "ListDeviceInstances",
        kind: "AWS::ApiOnly::DeviceFarmDeviceInstance",
    },
    OperationMapping {
        service: "device-farm",
        operation: "ListInstanceProfiles",
        kind: "AWS::ApiOnly::DeviceFarmInstanceProfile",
    },
    OperationMapping {
        service: "device-farm",
        operation: "ListTestGridProjects",
        kind: "AWS::ApiOnly::DeviceFarmTestGridProject",
    },
    OperationMapping {
        service: "device-farm",
        operation: "ListVPCEConfigurations",
        kind: "AWS::ApiOnly::DeviceFarmVPCEConfiguration",
    },
    OperationMapping {
        service: "devops-agent",
        operation: "ListAgentSpaces",
        kind: "AWS::ApiOnly::DevOpsAgentAgentSpace",
    },
    OperationMapping {
        service: "devops-agent",
        operation: "ListPrivateConnections",
        kind: "AWS::ApiOnly::DevOpsAgentPrivateConnection",
    },
];
