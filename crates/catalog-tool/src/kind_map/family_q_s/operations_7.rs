use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "sagemaker", operation: "ListDataQualityJobDefinitions", kind: "AWS::SageMaker::DataQualityJobDefinition" },
    OperationMapping { service: "sagemaker", operation: "ListDevices", kind: "AWS::SageMaker::Device" },
    OperationMapping { service: "sagemaker", operation: "ListEdgeDeploymentPlans", kind: "AWS::ApiOnly::SageMakerEdgeDeploymentPlan" },
    OperationMapping { service: "sagemaker", operation: "ListEdgePackagingJobs", kind: "AWS::ApiOnly::SageMakerEdgePackagingJob" },
    OperationMapping { service: "sagemaker", operation: "ListFlowDefinitions", kind: "AWS::ApiOnly::SageMakerFlowDefinition" },
    OperationMapping { service: "sagemaker", operation: "ListHubs", kind: "AWS::ApiOnly::SageMakerHub" },
    OperationMapping { service: "sagemaker", operation: "ListHumanTaskUis", kind: "AWS::ApiOnly::SageMakerHumanTaskUi" },
    OperationMapping { service: "sagemaker", operation: "ListHyperParameterTuningJobs", kind: "AWS::ApiOnly::SageMakerHyperParameterTuningJob" },
    OperationMapping { service: "sagemaker", operation: "ListInferenceExperiments", kind: "AWS::SageMaker::InferenceExperiment" },
    OperationMapping { service: "sagemaker", operation: "ListLabelingJobs", kind: "AWS::ApiOnly::SageMakerLabelingJob" },
    OperationMapping { service: "sagemaker", operation: "ListModelBiasJobDefinitions", kind: "AWS::SageMaker::ModelBiasJobDefinition" },
    OperationMapping { service: "sagemaker", operation: "ListModelCards", kind: "AWS::SageMaker::ModelCard" },
    OperationMapping { service: "sagemaker", operation: "ListModelExplainabilityJobDefinitions", kind: "AWS::SageMaker::ModelExplainabilityJobDefinition" },
    OperationMapping { service: "sagemaker", operation: "ListModelPackages", kind: "AWS::SageMaker::ModelPackage" },
    OperationMapping { service: "sagemaker", operation: "ListModelQualityJobDefinitions", kind: "AWS::SageMaker::ModelQualityJobDefinition" },
    OperationMapping { service: "sagemaker", operation: "ListMonitoringSchedules", kind: "AWS::SageMaker::MonitoringSchedule" },
    OperationMapping { service: "sagemaker", operation: "ListOptimizationJobs", kind: "AWS::ApiOnly::SageMakerOptimizationJob" },
    OperationMapping { service: "sagemaker", operation: "ListPartnerApps", kind: "AWS::SageMaker::PartnerApp" },
    OperationMapping { service: "sagemaker", operation: "ListProcessingJobs", kind: "AWS::ApiOnly::SageMakerProcessingJob" },
    OperationMapping { service: "sagemaker", operation: "ListProjects", kind: "AWS::SageMaker::Project" },
];
