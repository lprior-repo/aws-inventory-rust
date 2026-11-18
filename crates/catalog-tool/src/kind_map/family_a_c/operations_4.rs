use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "applicationcostprofiler",
        operation: "ListReportDefinitions",
        kind: "AWS::ApiOnly::ApplicationCostProfilerReportDefinition",
    },
    OperationMapping {
        service: "apprunner",
        operation: "DescribeAutoScalingConfiguration",
        kind: "AWS::AppRunner::AutoScalingConfiguration",
    },
    OperationMapping {
        service: "apprunner",
        operation: "DescribeObservabilityConfiguration",
        kind: "AWS::AppRunner::ObservabilityConfiguration",
    },
    OperationMapping {
        service: "apprunner",
        operation: "DescribeVpcConnector",
        kind: "AWS::AppRunner::VpcConnector",
    },
    OperationMapping {
        service: "apprunner",
        operation: "DescribeVpcIngressConnection",
        kind: "AWS::AppRunner::VpcIngressConnection",
    },
    OperationMapping {
        service: "apprunner",
        operation: "ListAutoScalingConfigurations",
        kind: "AWS::AppRunner::AutoScalingConfiguration",
    },
    OperationMapping {
        service: "apprunner",
        operation: "ListConnections",
        kind: "AWS::ApiOnly::AppRunnerConnection",
    },
    OperationMapping {
        service: "apprunner",
        operation: "ListObservabilityConfigurations",
        kind: "AWS::AppRunner::ObservabilityConfiguration",
    },
    OperationMapping {
        service: "apprunner",
        operation: "ListServices",
        kind: "AWS::AppRunner::Service",
    },
    OperationMapping {
        service: "apprunner",
        operation: "ListVpcConnectors",
        kind: "AWS::AppRunner::VpcConnector",
    },
    OperationMapping {
        service: "apprunner",
        operation: "ListVpcIngressConnections",
        kind: "AWS::AppRunner::VpcIngressConnection",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeAppBlockBuilders",
        kind: "AWS::AppStream::AppBlockBuilder",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeAppBlocks",
        kind: "AWS::AppStream::AppBlock",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeApplications",
        kind: "AWS::AppStream::Application",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeEntitlements",
        kind: "AWS::AppStream::Entitlement",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeFleets",
        kind: "AWS::AppStream::Fleet",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeImageBuilders",
        kind: "AWS::AppStream::ImageBuilder",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeImages",
        kind: "AWS::ApiOnly::AppStreamImage",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeStacks",
        kind: "AWS::AppStream::Stack",
    },
    OperationMapping {
        service: "appstream",
        operation: "DescribeUsageReportSubscriptions",
        kind: "AWS::ApiOnly::AppStreamUsageReportSubscription",
    },
];
