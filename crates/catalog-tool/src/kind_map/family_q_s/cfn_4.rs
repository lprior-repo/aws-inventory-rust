use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::SSM::Parameter"),
    CfnMapping::canonical("AWS::SSM::PatchBaseline"),
    CfnMapping::canonical("AWS::SSM::ResourceDataSync"),
    CfnMapping::canonical("AWS::SSMContacts::Contact"),
    CfnMapping::canonical("AWS::SSMContacts::Rotation"),
    CfnMapping::canonical("AWS::SSMIncidents::ReplicationSet"),
    CfnMapping::canonical("AWS::SSMIncidents::ResponsePlan"),
    CfnMapping::canonical("AWS::SSO::Application"),
    CfnMapping::canonical("AWS::SSO::Instance"),
    CfnMapping::canonical("AWS::SSO::PermissionSet"),
    CfnMapping::canonical("AWS::SageMaker::ClusterSchedulerConfig"),
    CfnMapping::canonical("AWS::SageMaker::CodeRepository"),
    CfnMapping::canonical("AWS::SageMaker::ComputeQuota"),
    CfnMapping::canonical("AWS::SageMaker::DataQualityJobDefinition"),
    CfnMapping::canonical("AWS::SageMaker::Device"),
    CfnMapping::canonical("AWS::SageMaker::DeviceFleet"),
    CfnMapping::canonical("AWS::SageMaker::Domain"),
    CfnMapping::canonical("AWS::SageMaker::Endpoint"),
    CfnMapping::canonical("AWS::SageMaker::EndpointConfig"),
    CfnMapping::canonical("AWS::SageMaker::FeatureGroup"),
];
