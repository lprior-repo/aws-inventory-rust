use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Redshift::ClusterSubnetGroup"),
    CfnMapping::canonical("AWS::Redshift::EndpointAccess"),
    CfnMapping::canonical("AWS::Redshift::EndpointAuthorization"),
    CfnMapping::canonical("AWS::Redshift::EventSubscription"),
    CfnMapping::canonical("AWS::Redshift::Integration"),
    CfnMapping::canonical("AWS::Redshift::ScheduledAction"),
    CfnMapping::canonical("AWS::Redshift::SnapshotCopyGrant"),
    CfnMapping::canonical("AWS::RedshiftServerless::Namespace"),
    CfnMapping::canonical("AWS::RedshiftServerless::Snapshot"),
    CfnMapping::canonical("AWS::RedshiftServerless::Workgroup"),
    CfnMapping::canonical("AWS::Rekognition::Collection"),
    CfnMapping::canonical("AWS::Rekognition::Project"),
    CfnMapping::canonical("AWS::Rekognition::StreamProcessor"),
    CfnMapping::canonical("AWS::ResilienceHub::App"),
    CfnMapping::canonical("AWS::ResilienceHub::ResiliencyPolicy"),
    CfnMapping::canonical("AWS::ResourceExplorer2::Index"),
    CfnMapping::canonical("AWS::ResourceExplorer2::View"),
    CfnMapping::canonical("AWS::ResourceGroups::Group"),
    CfnMapping::canonical("AWS::RolesAnywhere::CRL"),
    CfnMapping::canonical("AWS::RolesAnywhere::Profile"),
];
