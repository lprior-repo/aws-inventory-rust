use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "redshift",
        operation: "DescribeClusterSnapshots",
        kind: "AWS::ApiOnly::RedshiftSnapshot",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeClusterParameterGroups",
        kind: "AWS::Redshift::ClusterParameterGroup",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeClusterSecurityGroups",
        kind: "AWS::Redshift::ClusterSecurityGroup",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeClusterSubnetGroups",
        kind: "AWS::Redshift::ClusterSubnetGroup",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeHsmClientCertificates",
        kind: "AWS::ApiOnly::RedshiftHsmClientCertificate",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeHsmConfigurations",
        kind: "AWS::ApiOnly::RedshiftHsmConfiguration",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeEventSubscriptions",
        kind: "AWS::Redshift::EventSubscription",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeReservedNodes",
        kind: "AWS::ApiOnly::RedshiftReservedNode",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeEndpointAccess",
        kind: "AWS::Redshift::EndpointAccess",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeRedshiftIdcApplications",
        kind: "AWS::ApiOnly::RedshiftIdcApplication",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeIntegrations",
        kind: "AWS::Redshift::Integration",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeScheduledActions",
        kind: "AWS::Redshift::ScheduledAction",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeSnapshotCopyGrants",
        kind: "AWS::Redshift::SnapshotCopyGrant",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeSnapshotSchedules",
        kind: "AWS::ApiOnly::RedshiftSnapshotSchedule",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeUsageLimits",
        kind: "AWS::ApiOnly::RedshiftUsageLimit",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeInboundIntegrations",
        kind: "AWS::ApiOnly::RedshiftInboundIntegration",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeQev2IdcApplications",
        kind: "AWS::ApiOnly::RedshiftQev2IdcApplication",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeDataSharesForProducer",
        kind: "AWS::ApiOnly::RedshiftDataShare",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeEndpointAuthorization",
        kind: "AWS::Redshift::EndpointAuthorization",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeReservedNodeExchangeStatus",
        kind: "AWS::ApiOnly::RedshiftReservedNodeExchange",
    },
];
