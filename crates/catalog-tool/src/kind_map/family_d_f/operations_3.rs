use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "drs", operation: "DescribeLaunchConfigurationTemplates", kind: "AWS::DRS::LaunchConfigurationTemplate" },
    OperationMapping { service: "drs", operation: "DescribeRecoveryInstances", kind: "AWS::ApiOnly::DRSRecoveryInstance" },
    OperationMapping { service: "drs", operation: "DescribeReplicationConfigurationTemplates", kind: "AWS::DRS::ReplicationConfigurationTemplate" },
    OperationMapping { service: "drs", operation: "DescribeSourceNetworks", kind: "AWS::ApiOnly::DRSSourceNetwork" },
    OperationMapping { service: "drs", operation: "DescribeSourceServers", kind: "AWS::ApiOnly::DRSSourceServer" },
    OperationMapping { service: "drs", operation: "ListRecoveryPlans", kind: "AWS::ApiOnly::DRSRecoveryPlan" },
    OperationMapping { service: "drs", operation: "ListRecoveryPlanExecutions", kind: "AWS::ApiOnly::DRSRecoveryPlanExecution" },
    OperationMapping { service: "dsql", operation: "ListClusters", kind: "AWS::DSQL::Cluster" },
    OperationMapping { service: "dynamodb-streams", operation: "ListStreams", kind: "AWS::ApiOnly::DynamoDBStream" },
    OperationMapping { service: "ecr-public", operation: "DescribeRegistries", kind: "AWS::ApiOnly::ECRPublicRegistry" },
    OperationMapping { service: "ecr-public", operation: "DescribeRepositories", kind: "AWS::ECR::PublicRepository" },
    OperationMapping { service: "ecr", operation: "DescribeRepositories", kind: "AWS::ECR::Repository" },
    OperationMapping { service: "ecr", operation: "DescribePullThroughCacheRules", kind: "AWS::ECR::PullThroughCacheRule" },
    OperationMapping { service: "ecr", operation: "DescribeRepositoryCreationTemplates", kind: "AWS::ECR::RepositoryCreationTemplate" },
    OperationMapping { service: "ecr", operation: "ListReplicationBackfills", kind: "AWS::ApiOnly::ECRReplicationBackfill" },
    OperationMapping { service: "efs", operation: "DescribeFileSystems", kind: "AWS::EFS::FileSystem" },
    OperationMapping { service: "efs", operation: "DescribeAccessPoints", kind: "AWS::EFS::AccessPoint" },
    OperationMapping { service: "efs", operation: "DescribeReplicationConfigurations", kind: "AWS::ApiOnly::EFSReplicationConfiguration" },
    OperationMapping { service: "efs", operation: "DescribeMountTargets", kind: "AWS::EFS::MountTarget" },
    OperationMapping { service: "elastic-beanstalk", operation: "DescribeApplications", kind: "AWS::ElasticBeanstalk::Application" },
];
