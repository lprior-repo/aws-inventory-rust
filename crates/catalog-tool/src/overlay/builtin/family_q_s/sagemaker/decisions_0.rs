use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AddAssociation", Decision::Denied(DenyReason::Mutating)),
    ("AddTags", Decision::Denied(DenyReason::Mutating)),
    (
        "AssociateTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AttachClusterNodeNetworkInterface",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AttachClusterNodeVolume",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchAddClusterNodes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteClusterNodes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDescribeModelPackage",
        Decision::Deferred(
            "Required selectors ModelPackageArnList are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelPackageSummaries, BatchDescribeModelPackageErrorMap. Pinned purpose: This action batch describes a list of versioned model packages",
        ),
    ),
    (
        "BatchRebootClusterNodes",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchReplaceClusterNodes",
        Decision::Deferred(
            "Required selectors ClusterName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Successful, Failed, FailedNodeLogicalIds, SuccessfulNodeLogicalIds. Pinned purpose: Replaces specific nodes within a SageMaker HyperPod cluster with new hardware. BatchReplaceClusterNodes terminates the specified instances and provisions new replacement instances with the same configuration but fresh hardware. The Amazon Machine Ima",
        ),
    ),
    (
        "CreateAIBenchmarkJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAIRecommendationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAIWorkloadConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAction", Decision::Denied(DenyReason::Mutating)),
    ("CreateAlgorithm", Decision::Denied(DenyReason::Mutating)),
    ("CreateApp", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAppImageConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateArtifact", Decision::Denied(DenyReason::Mutating)),
    ("CreateAutoMLJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateAutoMLJobV2", Decision::Denied(DenyReason::Mutating)),
    ("CreateCluster", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateClusterExtension",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateClusterSchedulerConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCodeRepository",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCompilationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateComputeQuota", Decision::Denied(DenyReason::Mutating)),
    ("CreateContext", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateDataQualityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateDeviceFleet", Decision::Denied(DenyReason::Mutating)),
    ("CreateDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateEdgeDeploymentPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEdgeDeploymentStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEdgePackagingJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateEndpoint", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateEndpointConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateExperiment", Decision::Denied(DenyReason::Mutating)),
    ("CreateFeatureGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateFlowDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateHub", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateHubContentPresignedUrls",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateHubContentReference",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateHumanTaskUi", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateHyperParameterTuningJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateImage", Decision::Denied(DenyReason::Mutating)),
    ("CreateImageVersion", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateInferenceComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateInferenceRecommendationsJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateLabelingJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateMlflowApp", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateModel", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateModelBiasJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateModelCard", Decision::Denied(DenyReason::Mutating)),
];
