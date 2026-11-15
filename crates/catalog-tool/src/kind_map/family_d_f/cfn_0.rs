use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::DAX::Cluster"),
    CfnMapping::canonical("AWS::DAX::ParameterGroup"),
    CfnMapping::canonical("AWS::DAX::SubnetGroup"),
    CfnMapping::canonical("AWS::DLM::LifecyclePolicy"),
    CfnMapping::canonical("AWS::DMS::EventSubscription"),
    CfnMapping::canonical("AWS::DMS::InstanceProfile"),
    CfnMapping::canonical("AWS::DMS::MigrationProject"),
    CfnMapping::canonical("AWS::DMS::ReplicationInstance"),
    CfnMapping::canonical("AWS::DMS::ReplicationSubnetGroup"),
    CfnMapping::canonical("AWS::DRS::LaunchConfigurationTemplate"),
    CfnMapping::canonical("AWS::DRS::ReplicationConfigurationTemplate"),
    CfnMapping::canonical("AWS::DSQL::Cluster"),
    CfnMapping::canonical("AWS::DataBrew::Job"),
    CfnMapping::canonical("AWS::DataBrew::Project"),
    CfnMapping::canonical("AWS::DataBrew::Ruleset"),
    CfnMapping::canonical("AWS::DataBrew::Schedule"),
    CfnMapping::canonical("AWS::DataPipeline::Pipeline"),
];
