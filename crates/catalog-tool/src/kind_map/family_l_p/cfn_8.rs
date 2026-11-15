use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::PinpointEmail::ConfigurationSet"),
    CfnMapping::canonical("AWS::PinpointEmail::DedicatedIpPool"),
    CfnMapping::canonical("AWS::PinpointEmail::Identity"),
    CfnMapping::canonical("AWS::Pipes::Pipe"),
    CfnMapping::canonical("AWS::Proton::EnvironmentTemplate"),
    CfnMapping::canonical("AWS::Proton::ServiceTemplate"),
    CfnMapping::canonical("AWS::RefactorSpaces::Application"),
    CfnMapping::canonical("AWS::RefactorSpaces::Environment"),
    CfnMapping::canonical("AWS::RefactorSpaces::Service"),
    CfnMapping::canonical("AWS::SES::MailManagerAddonInstance"),
    CfnMapping::canonical("AWS::SES::MailManagerAddonSubscription"),
    CfnMapping::canonical("AWS::SES::MailManagerAddressList"),
    CfnMapping::canonical("AWS::SES::MailManagerArchive"),
    CfnMapping::canonical("AWS::SES::MailManagerIngressPoint"),
    CfnMapping::canonical("AWS::SES::MailManagerRelay"),
    CfnMapping::canonical("AWS::SES::MailManagerRuleSet"),
    CfnMapping::canonical("AWS::SES::MailManagerTrafficPolicy"),
    CfnMapping::canonical("AWS::SMSVOICE::ConfigurationSet"),
    CfnMapping::canonical("AWS::SMSVOICE::OptOutList"),
    CfnMapping::canonical("AWS::SMSVOICE::PhoneNumber"),
];
