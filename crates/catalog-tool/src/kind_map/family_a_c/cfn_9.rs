use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Bedrock::ApplicationInferenceProfile"),
    CfnMapping::canonical("AWS::Logs::LogAnomalyDetector"),
    CfnMapping::canonical("AWS::Logs::LogGroup"),
    CfnMapping::canonical("AWS::Logs::LogStream"),
    CfnMapping::canonical("AWS::Logs::MetricFilter"),
    CfnMapping::canonical("AWS::Logs::ResourcePolicy"),
    CfnMapping::canonical("AWS::Logs::ScheduledQuery"),
    CfnMapping::canonical("AWS::S3::Bucket"),
];
