use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::AmazonMQ::Broker"),
    CfnMapping::canonical("AWS::AmazonMQ::Configuration"),
    CfnMapping::canonical("AWS::HealthImaging::Datastore"),
    CfnMapping::canonical("AWS::LakeFormation::Resource"),
    CfnMapping::canonical("AWS::LakeFormation::Tag"),
    CfnMapping::canonical("AWS::Lambda::MicrovmImage"),
    CfnMapping::canonical("AWS::Lambda::NetworkConnector"),
    CfnMapping::canonical("AWS::Lambda::WebFunction"),
    CfnMapping::canonical("AWS::LaunchWizard::Deployment"),
    CfnMapping::canonical("AWS::Lex::Bot"),
    CfnMapping::canonical("AWS::LicenseManager::License"),
    CfnMapping::canonical("AWS::LicenseManager::LicenseAssetGroup"),
    CfnMapping::canonical("AWS::LicenseManager::LicenseConfiguration"),
    CfnMapping::canonical("AWS::LicenseManager::ReportGenerator"),
    CfnMapping::canonical("AWS::Lightsail::Alarm"),
    CfnMapping::canonical("AWS::Lightsail::Bucket"),
    CfnMapping::canonical("AWS::Lightsail::Certificate"),
    CfnMapping::canonical("AWS::Lightsail::DatabaseSnapshot"),
    CfnMapping::canonical("AWS::Lightsail::Disk"),
    CfnMapping::canonical("AWS::Lightsail::DiskSnapshot"),
];
