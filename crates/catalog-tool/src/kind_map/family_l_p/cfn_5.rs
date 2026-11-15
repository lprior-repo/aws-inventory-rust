use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::NetworkManager::GlobalNetwork"),
    CfnMapping::canonical("AWS::NetworkManager::Link"),
    CfnMapping::canonical("AWS::NetworkManager::Site"),
    CfnMapping::canonical("AWS::NetworkMonitor::Monitor"),
    CfnMapping::canonical("AWS::NetworkSecurityManager::Deployment"),
    CfnMapping::canonical("AWS::NetworkSecurityManager::Policy"),
    CfnMapping::canonical("AWS::NetworkSecurityManager::Rule"),
    CfnMapping::canonical("AWS::NetworkSecurityManager::Scope"),
    CfnMapping::canonical("AWS::Notifications::NotificationConfiguration"),
    CfnMapping::canonical("AWS::Notifications::NotificationHub"),
    CfnMapping::canonical("AWS::NotificationsContacts::EmailContact"),
    CfnMapping::canonical("AWS::NovaAct::WorkflowDefinition"),
    CfnMapping::canonical("AWS::ODB::CloudAutonomousVmCluster"),
    CfnMapping::canonical("AWS::ODB::CloudExadataInfrastructure"),
    CfnMapping::canonical("AWS::ODB::CloudVmCluster"),
    CfnMapping::canonical("AWS::ODB::OdbNetwork"),
    CfnMapping::canonical("AWS::ODB::OdbPeeringConnection"),
    CfnMapping::canonical("AWS::OSIS::Pipeline"),
    CfnMapping::canonical("AWS::OSIS::PipelineEndpoint"),
    CfnMapping::canonical("AWS::Oam::Link"),
];
