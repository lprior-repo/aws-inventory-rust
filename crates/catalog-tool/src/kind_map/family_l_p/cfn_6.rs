use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Oam::Sink"),
    CfnMapping::canonical("AWS::ObservabilityAdmin::DatasetIntegration"),
    CfnMapping::canonical("AWS::ObservabilityAdmin::OrganizationCentralizationRule"),
    CfnMapping::canonical("AWS::ObservabilityAdmin::OrganizationTelemetryRule"),
    CfnMapping::canonical("AWS::ObservabilityAdmin::S3TableIntegration"),
    CfnMapping::canonical("AWS::ObservabilityAdmin::TelemetryPipelines"),
    CfnMapping::canonical("AWS::ObservabilityAdmin::TelemetryRule"),
    CfnMapping::canonical("AWS::Omics::AnnotationStore"),
    CfnMapping::canonical("AWS::Omics::ReferenceStore"),
    CfnMapping::canonical("AWS::Omics::RunCache"),
    CfnMapping::canonical("AWS::Omics::RunGroup"),
    CfnMapping::canonical("AWS::Omics::SequenceStore"),
    CfnMapping::canonical("AWS::Omics::VariantStore"),
    CfnMapping::canonical("AWS::Omics::Workflow"),
    CfnMapping::canonical("AWS::OpenSearchServerless::Collection"),
    CfnMapping::canonical("AWS::OpenSearchServerless::CollectionGroup"),
    CfnMapping::canonical("AWS::OpenSearchServerless::VpcEndpoint"),
    CfnMapping::canonical("AWS::OpenSearchService::Application"),
    CfnMapping::canonical("AWS::OpenSearchService::Domain"),
    CfnMapping::canonical("AWS::Outposts::Outpost"),
];
