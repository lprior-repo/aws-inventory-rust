use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::BedrockAgentCore::Runtime"),
    CfnMapping::canonical("AWS::BedrockAgentCore::WorkloadIdentity"),
    CfnMapping::canonical("AWS::Billing::BillingView"),
    CfnMapping::canonical("AWS::BillingConductor::BillingGroup"),
    CfnMapping::canonical("AWS::BillingConductor::CustomLineItem"),
    CfnMapping::canonical("AWS::BillingConductor::PricingPlan"),
    CfnMapping::canonical("AWS::BillingConductor::PricingRule"),
    CfnMapping::canonical("AWS::Budgets::Budget"),
    CfnMapping::canonical("AWS::CUR::ReportDefinition"),
    CfnMapping::canonical("AWS::Cases::Domain"),
    CfnMapping::canonical("AWS::CertificateManager::Certificate"),
    CfnMapping::canonical("AWS::Chatbot::CustomAction"),
    CfnMapping::canonical("AWS::Chatbot::MicrosoftTeamsChannelConfiguration"),
    CfnMapping::canonical("AWS::Chatbot::SlackChannelConfiguration"),
    CfnMapping::canonical("AWS::Chime::SipMediaApplication"),
    CfnMapping::canonical("AWS::Chime::VoiceConnector"),
    CfnMapping::canonical("AWS::CleanRooms::Collaboration"),
    CfnMapping::canonical("AWS::CleanRooms::ConfiguredTable"),
    CfnMapping::canonical("AWS::CleanRooms::Membership"),
    CfnMapping::canonical("AWS::CloudFormation::Stack"),
];
