use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::S3::StorageLens"),
    CfnMapping::canonical("AWS::S3::StorageLensGroup"),
    CfnMapping::canonical("AWS::S3ObjectLambda::AccessPoint"),
    CfnMapping::canonical("AWS::S3Outposts::Bucket"),
    CfnMapping::canonical("AWS::S3Outposts::Endpoint"),
    CfnMapping::canonical("AWS::S3Tables::Table"),
    CfnMapping::canonical("AWS::S3Tables::TableBucket"),
    CfnMapping::canonical("AWS::S3Vectors::Index"),
    CfnMapping::canonical("AWS::S3Vectors::VectorBucket"),
    CfnMapping::canonical("AWS::SES::ConfigurationSet"),
    CfnMapping::canonical("AWS::SES::ContactList"),
    CfnMapping::canonical("AWS::SES::DedicatedIpPool"),
    CfnMapping::canonical("AWS::SES::EmailIdentity"),
    CfnMapping::canonical("AWS::SES::MultiRegionEndpoint"),
    CfnMapping::canonical("AWS::SES::ReceiptFilter"),
    CfnMapping::canonical("AWS::SES::ReceiptRuleSet"),
    CfnMapping::canonical("AWS::SES::Template"),
    CfnMapping::canonical("AWS::SES::Tenant"),
    CfnMapping::canonical("AWS::SSM::Association"),
    CfnMapping::canonical("AWS::SSM::MaintenanceWindow"),
];
