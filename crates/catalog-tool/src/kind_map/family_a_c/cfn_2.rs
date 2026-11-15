use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::AppRunner::VpcConnector"),
    CfnMapping::canonical("AWS::AppRunner::VpcIngressConnection"),
    CfnMapping::canonical("AWS::AppStream::AppBlock"),
    CfnMapping::canonical("AWS::AppStream::AppBlockBuilder"),
    CfnMapping::canonical("AWS::AppStream::Application"),
    CfnMapping::canonical("AWS::AppStream::Entitlement"),
    CfnMapping::canonical("AWS::AppStream::Fleet"),
    CfnMapping::canonical("AWS::AppStream::ImageBuilder"),
    CfnMapping::canonical("AWS::AppStream::Stack"),
    CfnMapping::canonical("AWS::AppSync::Api"),
    CfnMapping::canonical("AWS::AppSync::ChannelNamespace"),
    CfnMapping::canonical("AWS::AppSync::DomainName"),
    CfnMapping::canonical("AWS::AppSync::GraphQLApi"),
    CfnMapping::canonical("AWS::ApplicationInsights::Application"),
    CfnMapping::canonical("AWS::ApplicationSignals::ServiceLevelObjective"),
    CfnMapping::canonical("AWS::Athena::CapacityReservation"),
    CfnMapping::canonical("AWS::Athena::DataCatalog"),
    CfnMapping::canonical("AWS::Athena::WorkGroup"),
    CfnMapping::canonical("AWS::AuditManager::Assessment"),
    CfnMapping::canonical("AWS::AutoScaling::AutoScalingGroup"),
];
