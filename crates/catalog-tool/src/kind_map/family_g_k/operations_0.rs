use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "gamelift",
        operation: "ListAliases",
        kind: "AWS::GameLift::Alias",
    },
    OperationMapping {
        service: "gamelift",
        operation: "ListBuilds",
        kind: "AWS::GameLift::Build",
    },
    OperationMapping {
        service: "gamelift",
        operation: "DescribeFleetAttributes",
        kind: "AWS::GameLift::Fleet",
    },
    OperationMapping {
        service: "gamelift",
        operation: "ListGameServerGroups",
        kind: "AWS::GameLift::GameServerGroup",
    },
    OperationMapping {
        service: "gamelift",
        operation: "DescribeAlias",
        kind: "AWS::GameLift::Alias",
    },
    OperationMapping {
        service: "gamelift",
        operation: "DescribeBuild",
        kind: "AWS::GameLift::Build",
    },
    OperationMapping {
        service: "gameliftstreams",
        operation: "ListApplications",
        kind: "AWS::ApiOnly::GameLiftStreamsApplication",
    },
    OperationMapping {
        service: "gameliftstreams",
        operation: "ListStreamGroups",
        kind: "AWS::ApiOnly::GameLiftStreamsStreamGroup",
    },
    OperationMapping {
        service: "glacier",
        operation: "ListVaults",
        kind: "AWS::ApiOnly::GlacierVault",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListAccelerators",
        kind: "AWS::GlobalAccelerator::Accelerator",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListCrossAccountAttachments",
        kind: "AWS::GlobalAccelerator::CrossAccountAttachment",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListCustomRoutingAccelerators",
        kind: "AWS::ApiOnly::GlobalAcceleratorCustomRoutingAccelerator",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "DescribeAccelerator",
        kind: "AWS::GlobalAccelerator::Accelerator",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListListeners",
        kind: "AWS::GlobalAccelerator::Listener",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListEndpointGroups",
        kind: "AWS::GlobalAccelerator::EndpointGroup",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListCustomRoutingListeners",
        kind: "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListCustomRoutingEndpointGroups",
        kind: "AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup",
    },
    OperationMapping {
        service: "glue",
        operation: "ListCrawlers",
        kind: "AWS::Glue::Crawler",
    },
    OperationMapping {
        service: "glue",
        operation: "ListJobs",
        kind: "AWS::Glue::Job",
    },
    OperationMapping {
        service: "glue",
        operation: "ListRegistries",
        kind: "AWS::Glue::Registry",
    },
    OperationMapping {
        service: "glue",
        operation: "ListSchemas",
        kind: "AWS::Glue::Schema",
    },
    OperationMapping {
        service: "glue",
        operation: "ListWorkflows",
        kind: "AWS::Glue::Workflow",
    },
    OperationMapping {
        service: "greengrass",
        operation: "ListGroups",
        kind: "AWS::Greengrass::Group",
    },
    OperationMapping {
        service: "greengrass",
        operation: "ListCoreDefinitions",
        kind: "AWS::Greengrass::CoreDefinition",
    },
];
