use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "devops-guru", operation: "ListNotificationChannels", kind: "AWS::DevOpsGuru::NotificationChannel" },
    OperationMapping { service: "direct-connect", operation: "DescribeConnections", kind: "AWS::DirectConnect::Connection" },
    OperationMapping { service: "direct-connect", operation: "DescribeDirectConnectGateways", kind: "AWS::DirectConnect::DirectConnectGateway" },
    OperationMapping { service: "direct-connect", operation: "DescribeInterconnects", kind: "AWS::ApiOnly::DirectConnectInterconnect" },
    OperationMapping { service: "direct-connect", operation: "DescribeLags", kind: "AWS::DirectConnect::Lag" },
    OperationMapping { service: "direct-connect", operation: "ListResiliencyGroups", kind: "AWS::ApiOnly::DirectConnectResiliencyGroup" },
    OperationMapping { service: "directory-service", operation: "DescribeSnapshots", kind: "AWS::ApiOnly::DirectoryServiceSnapshot" },
    OperationMapping { service: "directory-service", operation: "DescribeTrusts", kind: "AWS::ApiOnly::DirectoryServiceTrust" },
    OperationMapping { service: "directory-service", operation: "ListADAssessments", kind: "AWS::ApiOnly::DirectoryServiceAssessment" },
    OperationMapping { service: "directory-service", operation: "ListLogSubscriptions", kind: "AWS::ApiOnly::DirectoryServiceLogSubscription" },
    OperationMapping { service: "dlm", operation: "GetLifecyclePolicies", kind: "AWS::DLM::LifecyclePolicy" },
    OperationMapping { service: "docdb-elastic", operation: "ListClusters", kind: "AWS::DocDBElastic::Cluster" },
    OperationMapping { service: "docdb-elastic", operation: "ListClusterSnapshots", kind: "AWS::ApiOnly::DocDBElasticClusterSnapshot" },
    OperationMapping { service: "docdb", operation: "DescribeDBClusterParameterGroups", kind: "AWS::DocDB::DBClusterParameterGroup" },
    OperationMapping { service: "docdb", operation: "DescribeDBClusterSnapshots", kind: "AWS::ApiOnly::DocDBDBClusterSnapshot" },
    OperationMapping { service: "docdb", operation: "DescribeDBClusters", kind: "AWS::DocDB::DBCluster" },
    OperationMapping { service: "docdb", operation: "DescribeDBSubnetGroups", kind: "AWS::DocDB::DBSubnetGroup" },
    OperationMapping { service: "docdb", operation: "DescribeEventSubscriptions", kind: "AWS::DocDB::EventSubscription" },
    OperationMapping { service: "docdb", operation: "DescribeGlobalClusters", kind: "AWS::DocDB::GlobalCluster" },
    OperationMapping { service: "drs", operation: "DescribeJobs", kind: "AWS::ApiOnly::DRSJob" },
];
