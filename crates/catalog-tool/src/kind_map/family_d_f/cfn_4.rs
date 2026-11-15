use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::FIS::ExperimentTemplate"),
    CfnMapping::canonical("AWS::FMS::Policy"),
    CfnMapping::canonical("AWS::FMS::ResourceSet"),
    CfnMapping::canonical("AWS::FSx::DataRepositoryAssociation"),
    CfnMapping::canonical("AWS::FSx::FileCache"),
    CfnMapping::canonical("AWS::FSx::FileSystem"),
    CfnMapping::canonical("AWS::FSx::S3AccessPointAttachment"),
    CfnMapping::canonical("AWS::FSx::Snapshot"),
    CfnMapping::canonical("AWS::FSx::StorageVirtualMachine"),
    CfnMapping::canonical("AWS::FSx::Volume"),
    CfnMapping::canonical("AWS::Forecast::Dataset"),
    CfnMapping::canonical("AWS::Forecast::DatasetGroup"),
    CfnMapping::canonical("AWS::FraudDetector::Detector"),
    CfnMapping::canonical("AWS::FraudDetector::EntityType"),
    CfnMapping::canonical("AWS::FraudDetector::EventType"),
    CfnMapping::canonical("AWS::FraudDetector::Label"),
    CfnMapping::canonical("AWS::FraudDetector::List"),
    CfnMapping::canonical("AWS::FraudDetector::Outcome"),
];
