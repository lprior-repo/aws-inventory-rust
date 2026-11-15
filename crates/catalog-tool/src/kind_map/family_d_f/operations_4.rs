use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "elastic-beanstalk", operation: "DescribeApplicationVersions", kind: "AWS::ElasticBeanstalk::ApplicationVersion" },
    OperationMapping { service: "elastic-beanstalk", operation: "DescribeEnvironments", kind: "AWS::ElasticBeanstalk::Environment" },
    OperationMapping { service: "elastic-load-balancing", operation: "DescribeLoadBalancers", kind: "AWS::ElasticLoadBalancing::LoadBalancer" },
    OperationMapping { service: "elasticache", operation: "DescribeCacheClusters", kind: "AWS::ElastiCache::CacheCluster" },
    OperationMapping { service: "elasticache", operation: "DescribeCacheParameterGroups", kind: "AWS::ElastiCache::ParameterGroup" },
    OperationMapping { service: "elasticache", operation: "DescribeCacheSecurityGroups", kind: "AWS::ElastiCache::SecurityGroup" },
    OperationMapping { service: "elasticache", operation: "DescribeCacheSubnetGroups", kind: "AWS::ElastiCache::SubnetGroup" },
    OperationMapping { service: "elasticache", operation: "DescribeGlobalReplicationGroups", kind: "AWS::ElastiCache::GlobalReplicationGroup" },
    OperationMapping { service: "elasticache", operation: "DescribeReplicationGroups", kind: "AWS::ElastiCache::ReplicationGroup" },
    OperationMapping { service: "elasticache", operation: "DescribeServerlessCaches", kind: "AWS::ElastiCache::ServerlessCache" },
    OperationMapping { service: "elasticache", operation: "DescribeServerlessCacheSnapshots", kind: "AWS::ApiOnly::ElastiCacheServerlessCacheSnapshot" },
    OperationMapping { service: "elasticache", operation: "DescribeSnapshots", kind: "AWS::ApiOnly::ElastiCacheSnapshot" },
    OperationMapping { service: "elasticache", operation: "DescribeUserGroups", kind: "AWS::ElastiCache::UserGroup" },
    OperationMapping { service: "elasticache", operation: "DescribeUsers", kind: "AWS::ElastiCache::User" },
    OperationMapping { service: "elasticsearch-service", operation: "ListDomainNames", kind: "AWS::Elasticsearch::Domain" },
    OperationMapping { service: "elasticsearch-service", operation: "ListVpcEndpoints", kind: "AWS::ApiOnly::ElasticsearchVpcEndpoint" },
    OperationMapping { service: "elasticsearch-service", operation: "DescribeInboundCrossClusterSearchConnections", kind: "AWS::ApiOnly::ElasticsearchInboundCrossClusterConnection" },
    OperationMapping { service: "elasticsearch-service", operation: "DescribeOutboundCrossClusterSearchConnections", kind: "AWS::ApiOnly::ElasticsearchOutboundCrossClusterConnection" },
    OperationMapping { service: "elasticsearch-service", operation: "DescribePackages", kind: "AWS::ApiOnly::ElasticsearchPackage" },
    OperationMapping { service: "elementalinference", operation: "ListDictionaries", kind: "AWS::ApiOnly::ElementalInferenceDictionary" },
];
