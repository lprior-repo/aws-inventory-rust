use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::EventSchemas::Discoverer"),
    CfnMapping::canonical("AWS::EventSchemas::Registry"),
    CfnMapping::canonical("AWS::EventSchemas::Schema"),
    CfnMapping::canonical("AWS::QBusiness::Application"),
    CfnMapping::canonical("AWS::QBusiness::Retriever"),
    CfnMapping::canonical("AWS::QuickSight::Analysis"),
    CfnMapping::canonical("AWS::QuickSight::CustomPermissions"),
    CfnMapping::canonical("AWS::QuickSight::Dashboard"),
    CfnMapping::canonical("AWS::QuickSight::DataSet"),
    CfnMapping::canonical("AWS::QuickSight::DataSource"),
    CfnMapping::canonical("AWS::QuickSight::Folder"),
    CfnMapping::canonical("AWS::QuickSight::Theme"),
    CfnMapping::canonical("AWS::QuickSight::VPCConnection"),
    CfnMapping::canonical("AWS::RAM::ResourceShare"),
    CfnMapping::canonical("AWS::RTBFabric::RequesterGateway"),
    CfnMapping::canonical("AWS::RTBFabric::ResponderGateway"),
    CfnMapping::canonical("AWS::RUM::AppMonitor"),
    CfnMapping::canonical("AWS::Redshift::Cluster"),
    CfnMapping::canonical("AWS::Redshift::ClusterParameterGroup"),
    CfnMapping::canonical("AWS::Redshift::ClusterSecurityGroup"),
];
