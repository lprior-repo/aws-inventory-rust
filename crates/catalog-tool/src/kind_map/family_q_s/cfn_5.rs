use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::SageMaker::Image"),
    CfnMapping::canonical("AWS::SageMaker::InferenceComponent"),
    CfnMapping::canonical("AWS::SageMaker::InferenceExperiment"),
    CfnMapping::canonical("AWS::SageMaker::MlflowTrackingServer"),
    CfnMapping::canonical("AWS::SageMaker::Model"),
    CfnMapping::canonical("AWS::SageMaker::ModelBiasJobDefinition"),
    CfnMapping::canonical("AWS::SageMaker::ModelCard"),
    CfnMapping::canonical("AWS::SageMaker::ModelExplainabilityJobDefinition"),
    CfnMapping::canonical("AWS::SageMaker::ModelPackage"),
    CfnMapping::canonical("AWS::SageMaker::ModelPackageGroup"),
    CfnMapping::canonical("AWS::SageMaker::ModelQualityJobDefinition"),
    CfnMapping::canonical("AWS::SageMaker::MonitoringSchedule"),
    CfnMapping::canonical("AWS::SageMaker::NotebookInstance"),
    CfnMapping::canonical("AWS::SageMaker::NotebookInstanceLifecycleConfig"),
    CfnMapping::canonical("AWS::SageMaker::PartnerApp"),
    CfnMapping::canonical("AWS::SageMaker::Pipeline"),
    CfnMapping::canonical("AWS::SageMaker::Project"),
    CfnMapping::canonical("AWS::SageMaker::Space"),
    CfnMapping::canonical("AWS::SageMaker::StudioLifecycleConfig"),
    CfnMapping::canonical("AWS::SageMaker::UserProfile"),
];
