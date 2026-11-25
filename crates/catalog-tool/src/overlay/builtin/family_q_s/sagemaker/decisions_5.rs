use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "RenderUiTemplate",
        Decision::Deferred(
            "Required selectors Task, RoleArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RenderedContent, Errors. Pinned purpose: Renders the UI template so that you can preview the worker's experience.",
        ),
    ),
    (
        "RetryPipelineExecution",
        Decision::Deferred(
            "Required selectors PipelineExecutionArn, ClientRequestToken are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineExecutionArn. Pinned purpose: Retry the execution of the pipeline.",
        ),
    ),
    (
        "Search",
        Decision::Deferred(
            "Required selectors Resource are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Results, NextToken, TotalHits. Pinned purpose: Finds SageMaker resources that match a search query. Matching resources are returned as a list of SearchRecord objects in the response. You can sort the search results by any resource property in a ascending or descending order. You can query against",
        ),
    ),
    (
        "SearchTrainingPlanOfferings",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members TrainingPlanOfferings, TrainingPlanExtensionOfferings. Pinned request/ownership contract: Searches for available training plan offerings based on specified criteria. Users search for available plan offerings based on their requirements (e.g., instance type, count, start time, duration). And then, they create a plan that best matches their needs using the ID of the plan offering they want to use. For more information about how to reserve GPU capacity for your SageMaker training jobs or SageMaker HyperPod clusters using Amazon SageMaker",
        ),
    ),
    (
        "SendPipelineExecutionStepFailure",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SendPipelineExecutionStepSuccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartClusterHealthCheck",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartEdgeDeploymentStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartPipelineExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartSession", Decision::Denied(DenyReason::Mutating)),
    ("StopAIBenchmarkJob", Decision::Denied(DenyReason::Mutating)),
    (
        "StopAIRecommendationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StopAutoMLJob", Decision::Denied(DenyReason::Mutating)),
    ("StopCompilationJob", Decision::Denied(DenyReason::Mutating)),
    (
        "StopEdgeDeploymentStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopEdgePackagingJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopHyperParameterTuningJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopInferenceRecommendationsJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StopJob", Decision::Denied(DenyReason::Mutating)),
    ("StopLabelingJob", Decision::Denied(DenyReason::Mutating)),
    (
        "StopMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopOptimizationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopPipelineExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StopProcessingJob", Decision::Denied(DenyReason::Mutating)),
    ("StopTrainingJob", Decision::Denied(DenyReason::Mutating)),
    (
        "StopTrainingSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StopTransformJob", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAction", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAppImageConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateArtifact", Decision::Denied(DenyReason::Mutating)),
    ("UpdateCluster", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateClusterExtension",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateClusterSchedulerConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateClusterSoftware",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateCodeRepository",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateComputeQuota", Decision::Denied(DenyReason::Mutating)),
    ("UpdateContext", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDeviceFleet", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDevices", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDomain", Decision::Denied(DenyReason::Mutating)),
    ("UpdateEndpoint", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateEndpointWeightsAndCapacities",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateExperiment", Decision::Denied(DenyReason::Mutating)),
    ("UpdateFeatureGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFeatureMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateHub", Decision::Denied(DenyReason::Mutating)),
    ("UpdateHubContent", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateHubContentReference",
        Decision::Denied(DenyReason::Mutating),
    ),
];
