use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "sagemaker", operation: "ListNotebookInstanceLifecycleConfigs", kind: "AWS::SageMaker::NotebookInstanceLifecycleConfig" },
    OperationMapping { service: "sagemaker", operation: "ListDomains", kind: "AWS::SageMaker::Domain" },
    OperationMapping { service: "sagemaker", operation: "ListUserProfiles", kind: "AWS::SageMaker::UserProfile" },
    OperationMapping { service: "sagemaker", operation: "ListSpaces", kind: "AWS::SageMaker::Space" },
    OperationMapping { service: "sagemaker", operation: "ListFeatureGroups", kind: "AWS::SageMaker::FeatureGroup" },
    OperationMapping { service: "sagemaker", operation: "ListModelPackageGroups", kind: "AWS::SageMaker::ModelPackageGroup" },
    OperationMapping { service: "sagemaker", operation: "ListPipelines", kind: "AWS::SageMaker::Pipeline" },
    OperationMapping { service: "sagemaker", operation: "ListExperiments", kind: "AWS::ApiOnly::SageMakerExperiment" },
    OperationMapping { service: "sagemaker", operation: "ListTrials", kind: "AWS::ApiOnly::SageMakerTrial" },
    OperationMapping { service: "sagemaker", operation: "ListImages", kind: "AWS::SageMaker::Image" },
    OperationMapping { service: "sagemaker", operation: "ListClusters", kind: "AWS::ApiOnly::SageMakerCluster" },
    OperationMapping { service: "sagemaker", operation: "ListDeviceFleets", kind: "AWS::SageMaker::DeviceFleet" },
    OperationMapping { service: "sagemaker", operation: "ListMlflowTrackingServers", kind: "AWS::SageMaker::MlflowTrackingServer" },
    OperationMapping { service: "sagemaker", operation: "ListMlflowApps", kind: "AWS::ApiOnly::SageMakerMlflowApp" },
    OperationMapping { service: "sagemaker", operation: "ListInferenceComponents", kind: "AWS::SageMaker::InferenceComponent" },
    OperationMapping { service: "sagemaker", operation: "ListAlgorithms", kind: "AWS::ApiOnly::SageMakerAlgorithm" },
    OperationMapping { service: "sagemaker", operation: "ListAutoMLJobs", kind: "AWS::ApiOnly::SageMakerAutoMLJob" },
    OperationMapping { service: "sagemaker", operation: "ListClusterSchedulerConfigs", kind: "AWS::SageMaker::ClusterSchedulerConfig" },
    OperationMapping { service: "sagemaker", operation: "ListCompilationJobs", kind: "AWS::ApiOnly::SageMakerCompilationJob" },
    OperationMapping { service: "sagemaker", operation: "ListComputeQuotas", kind: "AWS::SageMaker::ComputeQuota" },
];
