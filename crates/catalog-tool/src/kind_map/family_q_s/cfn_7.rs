use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Shield::Protection"),
    CfnMapping::canonical("AWS::Shield::ProtectionGroup"),
    CfnMapping::canonical("AWS::Signer::SigningProfile"),
    CfnMapping::canonical("AWS::SupportApp::SlackChannelConfiguration"),
    CfnMapping::canonical("AWS::SupportApp::SlackWorkspaceConfiguration"),
    CfnMapping::canonical("AWS::Synthetics::Group"),
    CfnMapping::canonical("AWS::SystemsManagerSAP::Application"),
    CfnMapping::canonical("AWS::Wisdom::Assistant"),
    CfnMapping::canonical("AWS::Wisdom::AssistantAssociation"),
    CfnMapping::canonical("AWS::Wisdom::KnowledgeBase"),
    CfnMapping::canonical("AWS::Route53RecoveryReadiness::Cell"),
    CfnMapping::canonical("AWS::Route53RecoveryReadiness::ReadinessCheck"),
    CfnMapping::canonical("AWS::Route53RecoveryReadiness::RecoveryGroup"),
    CfnMapping::canonical("AWS::Route53RecoveryReadiness::ResourceSet"),
    CfnMapping::canonical("AWS::S3::MultiRegionAccessPoint"),
    CfnMapping::canonical("AWS::SNS::Subscription"),
    CfnMapping::canonical("AWS::SQS::Queue"),
];
