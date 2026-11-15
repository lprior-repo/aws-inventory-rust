use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Cognito::IdentityPool"),
    CfnMapping::canonical("AWS::Cognito::UserPool"),
    CfnMapping::canonical("AWS::Cognito::UserPoolClient"),
    CfnMapping::canonical("AWS::Comprehend::DocumentClassifier"),
    CfnMapping::canonical("AWS::Comprehend::EntityRecognizer"),
    CfnMapping::canonical("AWS::Comprehend::Flywheel"),
    CfnMapping::canonical("AWS::Config::AggregationAuthorization"),
    CfnMapping::canonical("AWS::Config::ConfigRule"),
    CfnMapping::canonical("AWS::Config::ConfigurationAggregator"),
    CfnMapping::canonical("AWS::Config::ConfigurationRecorder"),
    CfnMapping::canonical("AWS::Config::ConformancePack"),
    CfnMapping::canonical("AWS::Config::DeliveryChannel"),
    CfnMapping::canonical("AWS::Config::OrganizationConfigRule"),
    CfnMapping::canonical("AWS::Config::OrganizationConformancePack"),
    CfnMapping::canonical("AWS::Config::StoredQuery"),
    CfnMapping::canonical("AWS::Connect::ContactFlow"),
    CfnMapping::canonical("AWS::Connect::ContactFlowModule"),
    CfnMapping::canonical("AWS::Connect::HoursOfOperation"),
    CfnMapping::canonical("AWS::Connect::Instance"),
    CfnMapping::canonical("AWS::Connect::PhoneNumber"),
];
