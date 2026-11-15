use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Textract::Adapter"),
    CfnMapping::canonical("AWS::Timestream::InfluxDBBackup"),
    CfnMapping::canonical("AWS::Timestream::InfluxDBCluster"),
    CfnMapping::canonical("AWS::Timestream::InfluxDBInstance"),
    CfnMapping::canonical("AWS::Transcribe::CallAnalyticsCategory"),
    CfnMapping::canonical("AWS::Transcribe::MedicalVocabulary"),
    CfnMapping::canonical("AWS::Transcribe::Vocabulary"),
    CfnMapping::canonical("AWS::Transcribe::VocabularyFilter"),
    CfnMapping::canonical("AWS::Transfer::Agreement"),
    CfnMapping::canonical("AWS::Transfer::Certificate"),
    CfnMapping::canonical("AWS::Transfer::Connector"),
    CfnMapping::canonical("AWS::Transfer::HostKey"),
    CfnMapping::canonical("AWS::Transfer::Profile"),
    CfnMapping::canonical("AWS::Transfer::Server"),
    CfnMapping::canonical("AWS::Transfer::User"),
    CfnMapping::canonical("AWS::Transfer::WebApp"),
    CfnMapping::canonical("AWS::Transfer::Workflow"),
    CfnMapping::canonical("AWS::Translate::ParallelData"),
    CfnMapping::canonical("AWS::Translate::Terminology"),
    CfnMapping::canonical("AWS::VerifiedPermissions::IdentitySource"),
];
