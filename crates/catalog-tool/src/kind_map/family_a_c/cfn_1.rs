use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::AppConfig::Environment"),
    CfnMapping::canonical("AWS::AppConfig::Extension"),
    CfnMapping::canonical("AWS::AppConfig::ExtensionAssociation"),
    CfnMapping::canonical("AWS::AppFabric::AppAuthorization"),
    CfnMapping::canonical("AWS::AppFabric::AppBundle"),
    CfnMapping::canonical("AWS::AppFabric::Ingestion"),
    CfnMapping::canonical("AWS::AppFlow::Connector"),
    CfnMapping::canonical("AWS::AppFlow::ConnectorProfile"),
    CfnMapping::canonical("AWS::AppFlow::Flow"),
    CfnMapping::canonical("AWS::AppIntegrations::Application"),
    CfnMapping::canonical("AWS::AppIntegrations::DataIntegration"),
    CfnMapping::canonical("AWS::AppIntegrations::EventIntegration"),
    CfnMapping::canonical("AWS::AppMesh::Mesh"),
    CfnMapping::canonical("AWS::AppMesh::VirtualGateway"),
    CfnMapping::canonical("AWS::AppMesh::VirtualNode"),
    CfnMapping::canonical("AWS::AppMesh::VirtualRouter"),
    CfnMapping::canonical("AWS::AppMesh::VirtualService"),
    CfnMapping::canonical("AWS::AppRunner::AutoScalingConfiguration"),
    CfnMapping::canonical("AWS::AppRunner::ObservabilityConfiguration"),
    CfnMapping::canonical("AWS::AppRunner::Service"),
];
