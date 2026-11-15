use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::DocDB::EventSubscription"),
    CfnMapping::canonical("AWS::DocDB::GlobalCluster"),
    CfnMapping::canonical("AWS::DocDBElastic::Cluster"),
    CfnMapping::canonical("AWS::ECR::PublicRepository"),
    CfnMapping::canonical("AWS::ECR::PullThroughCacheRule"),
    CfnMapping::canonical("AWS::ECR::Repository"),
    CfnMapping::canonical("AWS::ECR::RepositoryCreationTemplate"),
    CfnMapping::canonical("AWS::EFS::AccessPoint"),
    CfnMapping::canonical("AWS::EFS::FileSystem"),
    CfnMapping::canonical("AWS::EFS::MountTarget"),
    CfnMapping::canonical("AWS::EMR::Cluster"),
    CfnMapping::canonical("AWS::EMR::SecurityConfiguration"),
    CfnMapping::canonical("AWS::EMR::Studio"),
    CfnMapping::canonical("AWS::EMRContainers::VirtualCluster"),
    CfnMapping::canonical("AWS::EMRServerless::Application"),
    CfnMapping::canonical("AWS::EVS::Environment"),
    CfnMapping::canonical("AWS::ElastiCache::CacheCluster"),
    CfnMapping::canonical("AWS::ElastiCache::GlobalReplicationGroup"),
    CfnMapping::canonical("AWS::ElastiCache::ParameterGroup"),
    CfnMapping::canonical("AWS::ElastiCache::ReplicationGroup"),
];
