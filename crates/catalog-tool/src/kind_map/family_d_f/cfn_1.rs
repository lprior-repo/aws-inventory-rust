use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::DataSync::Agent"),
    CfnMapping::canonical("AWS::DataSync::Task"),
    CfnMapping::canonical("AWS::DataZone::DataSource"),
    CfnMapping::canonical("AWS::DataZone::Domain"),
    CfnMapping::canonical("AWS::DataZone::Environment"),
    CfnMapping::canonical("AWS::DataZone::EnvironmentProfile"),
    CfnMapping::canonical("AWS::DataZone::Project"),
    CfnMapping::canonical("AWS::Deadline::Farm"),
    CfnMapping::canonical("AWS::Deadline::Fleet"),
    CfnMapping::canonical("AWS::Deadline::LicenseEndpoint"),
    CfnMapping::canonical("AWS::Deadline::Monitor"),
    CfnMapping::canonical("AWS::Deadline::Queue"),
    CfnMapping::canonical("AWS::Detective::Graph"),
    CfnMapping::canonical("AWS::DevOpsGuru::NotificationChannel"),
    CfnMapping::canonical("AWS::DirectConnect::Connection"),
    CfnMapping::canonical("AWS::DirectConnect::DirectConnectGateway"),
    CfnMapping::canonical("AWS::DirectConnect::Lag"),
    CfnMapping::canonical("AWS::DocDB::DBCluster"),
    CfnMapping::canonical("AWS::DocDB::DBClusterParameterGroup"),
    CfnMapping::canonical("AWS::DocDB::DBSubnetGroup"),
];
