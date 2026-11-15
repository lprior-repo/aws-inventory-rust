use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::ElastiCache::SecurityGroup"),
    CfnMapping::canonical("AWS::ElastiCache::ServerlessCache"),
    CfnMapping::canonical("AWS::ElastiCache::SubnetGroup"),
    CfnMapping::canonical("AWS::ElastiCache::User"),
    CfnMapping::canonical("AWS::ElastiCache::UserGroup"),
    CfnMapping::canonical("AWS::ElasticBeanstalk::Application"),
    CfnMapping::canonical("AWS::ElasticBeanstalk::ApplicationVersion"),
    CfnMapping::canonical("AWS::ElasticBeanstalk::Environment"),
    CfnMapping::canonical("AWS::ElasticLoadBalancing::LoadBalancer"),
    CfnMapping::canonical("AWS::Elasticsearch::Domain"),
    CfnMapping::canonical("AWS::EntityResolution::IdMappingWorkflow"),
    CfnMapping::canonical("AWS::EntityResolution::IdNamespace"),
    CfnMapping::canonical("AWS::EntityResolution::MatchingWorkflow"),
    CfnMapping::canonical("AWS::EntityResolution::SchemaMapping"),
    CfnMapping::canonical("AWS::Events::ApiDestination"),
    CfnMapping::canonical("AWS::Events::Archive"),
    CfnMapping::canonical("AWS::Events::Connection"),
    CfnMapping::canonical("AWS::Events::Endpoint"),
    CfnMapping::canonical("AWS::Events::EventBus"),
    CfnMapping::canonical("AWS::Events::Rule"),
];
