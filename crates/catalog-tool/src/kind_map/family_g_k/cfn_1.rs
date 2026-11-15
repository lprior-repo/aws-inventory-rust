use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Greengrass::SubscriptionDefinition"),
    CfnMapping::canonical("AWS::GreengrassV2::ComponentVersion"),
    CfnMapping::canonical("AWS::GreengrassV2::Deployment"),
    CfnMapping::canonical("AWS::GroundStation::Config"),
    CfnMapping::canonical("AWS::GroundStation::DataflowEndpointGroup"),
    CfnMapping::canonical("AWS::GroundStation::MissionProfile"),
    CfnMapping::canonical("AWS::GuardDuty::Detector"),
    CfnMapping::canonical("AWS::GuardDuty::MalwareProtectionPlan"),
    CfnMapping::canonical("AWS::HealthLake::FHIRDatastore"),
    CfnMapping::canonical("AWS::IVS::Channel"),
    CfnMapping::canonical("AWS::IVS::EncoderConfiguration"),
    CfnMapping::canonical("AWS::IVS::PlaybackKeyPair"),
    CfnMapping::canonical("AWS::IVS::PlaybackRestrictionPolicy"),
    CfnMapping::canonical("AWS::IVS::RecordingConfiguration"),
    CfnMapping::canonical("AWS::IVS::Stage"),
    CfnMapping::canonical("AWS::IVS::StorageConfiguration"),
    CfnMapping::canonical("AWS::IVSChat::LoggingConfiguration"),
    CfnMapping::canonical("AWS::IVSChat::Room"),
    CfnMapping::canonical("AWS::ImageBuilder::ContainerRecipe"),
    CfnMapping::canonical("AWS::ImageBuilder::DistributionConfiguration"),
];
