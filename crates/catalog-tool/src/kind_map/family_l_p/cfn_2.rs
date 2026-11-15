use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::MPA::IdentitySource"),
    CfnMapping::canonical("AWS::MWAA::Environment"),
    CfnMapping::canonical("AWS::Macie::AllowList"),
    CfnMapping::canonical("AWS::Macie::CustomDataIdentifier"),
    CfnMapping::canonical("AWS::Macie::FindingsFilter"),
    CfnMapping::canonical("AWS::ManagedBlockchain::Accessor"),
    CfnMapping::canonical("AWS::ManagedBlockchain::Member"),
    CfnMapping::canonical("AWS::MediaConnect::Bridge"),
    CfnMapping::canonical("AWS::MediaConnect::Flow"),
    CfnMapping::canonical("AWS::MediaConnect::Gateway"),
    CfnMapping::canonical("AWS::MediaConnect::RouterInput"),
    CfnMapping::canonical("AWS::MediaConnect::RouterNetworkInterface"),
    CfnMapping::canonical("AWS::MediaConnect::RouterOutput"),
    CfnMapping::canonical("AWS::MediaConvert::Queue"),
    CfnMapping::canonical("AWS::MediaLive::Channel"),
    CfnMapping::canonical("AWS::MediaLive::CloudWatchAlarmTemplate"),
    CfnMapping::canonical("AWS::MediaLive::CloudWatchAlarmTemplateGroup"),
    CfnMapping::canonical("AWS::MediaLive::Cluster"),
    CfnMapping::canonical("AWS::MediaLive::EventBridgeRuleTemplate"),
    CfnMapping::canonical("AWS::MediaLive::EventBridgeRuleTemplateGroup"),
];
