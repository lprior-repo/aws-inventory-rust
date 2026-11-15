use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "sagemaker", operation: "ListResourceCatalogs", kind: "AWS::ApiOnly::SageMakerResourceCatalog" },
    OperationMapping { service: "sagemaker", operation: "ListStudioLifecycleConfigs", kind: "AWS::SageMaker::StudioLifecycleConfig" },
    OperationMapping { service: "sagemaker", operation: "ListTrainingJobs", kind: "AWS::ApiOnly::SageMakerTrainingJob" },
    OperationMapping { service: "sagemaker", operation: "ListTrainingPlans", kind: "AWS::ApiOnly::SageMakerTrainingPlan" },
    OperationMapping { service: "sagemaker", operation: "ListTrainingSessions", kind: "AWS::ApiOnly::SageMakerTrainingSession" },
    OperationMapping { service: "sagemaker", operation: "ListTransformJobs", kind: "AWS::ApiOnly::SageMakerTransformJob" },
    OperationMapping { service: "sagemaker", operation: "ListTrialComponents", kind: "AWS::ApiOnly::SageMakerTrialComponent" },
    OperationMapping { service: "sagemaker", operation: "ListWorkforces", kind: "AWS::SageMaker::Workforce" },
    OperationMapping { service: "sagemaker", operation: "ListWorkteams", kind: "AWS::SageMaker::Workteam" },
    OperationMapping { service: "sagemaker", operation: "ListActions", kind: "AWS::ApiOnly::SageMakerLineageAction" },
    OperationMapping { service: "sagemaker", operation: "ListArtifacts", kind: "AWS::ApiOnly::SageMakerLineageArtifact" },
    OperationMapping { service: "sagemaker", operation: "ListContexts", kind: "AWS::ApiOnly::SageMakerLineageContext" },
    OperationMapping { service: "sagemaker", operation: "ListAIBenchmarkJobs", kind: "AWS::ApiOnly::SageMakerAIBenchmarkJob" },
    OperationMapping { service: "sagemaker", operation: "ListAIRecommendationJobs", kind: "AWS::ApiOnly::SageMakerAIRecommendationJob" },
    OperationMapping { service: "sagemaker", operation: "ListAIWorkloadConfigs", kind: "AWS::ApiOnly::SageMakerAIWorkloadConfig" },
    OperationMapping { service: "sagemaker", operation: "ListAssociations", kind: "AWS::ApiOnly::SageMakerLineageAssociation" },
    OperationMapping { service: "sagemaker", operation: "ListCodeRepositories", kind: "AWS::SageMaker::CodeRepository" },
    OperationMapping { service: "sagemaker", operation: "ListInferenceRecommendationsJobs", kind: "AWS::ApiOnly::SageMakerInferenceRecommendationJob" },
    OperationMapping { service: "sagemaker-geospatial", operation: "ListEarthObservationJobs", kind: "AWS::ApiOnly::SageMakerEarthObservationJob" },
    OperationMapping { service: "sagemaker-geospatial", operation: "ListVectorEnrichmentJobs", kind: "AWS::ApiOnly::SageMakerVectorEnrichmentJob" },
];
