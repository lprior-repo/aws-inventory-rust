use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Lightsail::Distribution"),
    CfnMapping::canonical("AWS::Lightsail::Domain"),
    CfnMapping::canonical("AWS::Lightsail::Instance"),
    CfnMapping::canonical("AWS::Lightsail::InstanceSnapshot"),
    CfnMapping::canonical("AWS::Lightsail::LoadBalancer"),
    CfnMapping::canonical("AWS::Lightsail::StaticIp"),
    CfnMapping::canonical("AWS::Location::GeofenceCollection"),
    CfnMapping::canonical("AWS::Location::Map"),
    CfnMapping::canonical("AWS::Location::PlaceIndex"),
    CfnMapping::canonical("AWS::Location::RouteCalculator"),
    CfnMapping::canonical("AWS::Location::Tracker"),
    CfnMapping::canonical("AWS::LookoutEquipment::InferenceScheduler"),
    CfnMapping::canonical("AWS::M2::Application"),
    CfnMapping::canonical("AWS::M2::Environment"),
    CfnMapping::canonical("AWS::MGN::Application"),
    CfnMapping::canonical("AWS::MGN::Connector"),
    CfnMapping::canonical("AWS::MGN::LaunchConfigurationTemplate"),
    CfnMapping::canonical("AWS::MGN::ReplicationConfigurationTemplate"),
    CfnMapping::canonical("AWS::MGN::Wave"),
    CfnMapping::canonical("AWS::MPA::ApprovalTeam"),
];
