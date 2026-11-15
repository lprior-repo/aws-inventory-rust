use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Connect::Prompt"),
    CfnMapping::canonical("AWS::Connect::Queue"),
    CfnMapping::canonical("AWS::Connect::RoutingProfile"),
    CfnMapping::canonical("AWS::Connect::SecurityProfile"),
    CfnMapping::canonical("AWS::Connect::TrafficDistributionGroup"),
    CfnMapping::canonical("AWS::ConnectCampaigns::Campaign"),
    CfnMapping::canonical("AWS::ConnectCampaignsV2::Campaign"),
    CfnMapping::canonical("AWS::ControlTower::EnabledBaseline"),
    CfnMapping::canonical("AWS::ControlTower::EnabledControl"),
    CfnMapping::canonical("AWS::CustomerProfiles::Domain"),
    CfnMapping::canonical("AWS::Events::ApiDestination"),
    CfnMapping::canonical("AWS::Events::Archive"),
    CfnMapping::canonical("AWS::Events::Connection"),
    CfnMapping::canonical("AWS::Events::EventBus"),
    CfnMapping::canonical("AWS::Events::Rule"),
    CfnMapping::canonical("AWS::Logs::Delivery"),
    CfnMapping::canonical("AWS::Logs::DeliveryDestination"),
    CfnMapping::canonical("AWS::Logs::DeliverySource"),
    CfnMapping::canonical("AWS::Logs::Destination"),
    CfnMapping::canonical("AWS::Logs::Integration"),
];
