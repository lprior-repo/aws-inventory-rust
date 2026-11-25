use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("DeleteHub", Decision::Denied(DenyReason::Mutating)),
    ("DeleteHubContent", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteHubContentReference",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteHumanTaskUi", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteHyperParameterTuningJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteImage", Decision::Denied(DenyReason::Mutating)),
    ("DeleteImageVersion", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteInferenceComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteJob", Decision::Denied(DenyReason::Mutating)),
    ("DeleteMlflowApp", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteModel", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteModelBiasJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteModelCard", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteModelExplainabilityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteModelPackage", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteModelPackageGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteModelPackageGroupPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteModelQualityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteNotebookInstanceLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteOptimizationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeletePartnerApp", Decision::Denied(DenyReason::Mutating)),
    ("DeletePipeline", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteProcessingJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteProject", Decision::Denied(DenyReason::Mutating)),
    ("DeleteSpace", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteStudioLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTags", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTrainingJob", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTrainingSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTrial", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteUserProfile", Decision::Denied(DenyReason::Mutating)),
    ("DeleteWorkforce", Decision::Denied(DenyReason::Mutating)),
    ("DeleteWorkteam", Decision::Denied(DenyReason::Mutating)),
    ("DeregisterDevices", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeAIBenchmarkJob",
        Decision::Deferred(
            "Required selectors AIBenchmarkJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AIBenchmarkJobName, AIBenchmarkJobArn, AIBenchmarkJobStatus, FailureReason, BenchmarkTarget, OutputConfig, AIWorkloadConfigIdentifier, RoleArn, NetworkConfig, CreationTime, StartTime, EndTime, Tags. Pinned purpose: Returns details of an AI benchmark job, including its status, configuration, target endpoint, and timing information.",
        ),
    ),
    (
        "DescribeAIRecommendationJob",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "DescribeAIWorkloadConfig",
        Decision::Deferred(
            "Required selectors AIWorkloadConfigName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AIWorkloadConfigName, AIWorkloadConfigArn, DatasetConfig, AIWorkloadConfigs, Tags, CreationTime. Pinned purpose: Returns details of an AI workload configuration, including the dataset configuration, benchmark tool settings, tags, and creation time.",
        ),
    ),
    (
        "DescribeAction",
        Decision::Deferred(
            "Required selectors ActionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ActionName, ActionArn, Source, ActionType, Description, Status, Properties, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, MetadataProperties, LineageGroupArn. Pinned purpose: Describes an action.",
        ),
    ),
    (
        "DescribeAlgorithm",
        Decision::Deferred(
            "Required selectors AlgorithmName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AlgorithmName, AlgorithmArn, AlgorithmDescription, CreationTime, TrainingSpecification, InferenceSpecification, ValidationSpecification, AlgorithmStatus, AlgorithmStatusDetails, ProductId, CertifyForMarketplace. Pinned purpose: Returns a description of the specified algorithm that is in your account.",
        ),
    ),
    (
        "DescribeApp",
        Decision::Deferred(
            "Required selectors DomainId, AppType, AppName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AppArn, AppType, AppName, DomainId, UserProfileName, SpaceName, Status, EffectiveTrustedIdentityPropagationStatus, RecoveryMode, LastHealthCheckTimestamp, LastUserActivityTimestamp, CreationTime, FailureReason, ResourceSpec, BuiltInLifecycleConfigArn. Pinned purpose: Describes the app.",
        ),
    ),
    (
        "DescribeAppImageConfig",
        Decision::Deferred(
            "Required selectors AppImageConfigName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AppImageConfigArn, AppImageConfigName, CreationTime, LastModifiedTime, KernelGatewayImageConfig, JupyterLabAppImageConfig, CodeEditorAppImageConfig. Pinned purpose: Describes an AppImageConfig.",
        ),
    ),
    (
        "DescribeArtifact",
        Decision::Deferred(
            "Required selectors ArtifactArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ArtifactName, ArtifactArn, Source, ArtifactType, Properties, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, MetadataProperties, LineageGroupArn. Pinned purpose: Describes an artifact.",
        ),
    ),
    (
        "DescribeAutoMLJob",
        Decision::Deferred(
            "Required selectors AutoMLJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AutoMLJobName, AutoMLJobArn, InputDataConfig, OutputDataConfig, RoleArn, AutoMLJobObjective, ProblemType, AutoMLJobConfig, CreationTime, EndTime, LastModifiedTime, FailureReason, PartialFailureReasons, BestCandidate, AutoMLJobStatus, AutoMLJobSecondaryStatus, GenerateCandidateDefinitionsOnly, AutoMLJobArtifacts, ResolvedAttributes, ModelDeployConfig, ModelDeployResult. Pinned purpose: Returns information about an AutoML job created by calling CreateAutoMLJob . AutoML jobs created by calling CreateAutoMLJobV2 cannot be described by DescribeAutoMLJob .",
        ),
    ),
    (
        "DescribeAutoMLJobV2",
        Decision::Deferred(
            "Required selectors AutoMLJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AutoMLJobName, AutoMLJobArn, AutoMLJobInputDataConfig, OutputDataConfig, RoleArn, AutoMLJobObjective, AutoMLProblemTypeConfig, AutoMLProblemTypeConfigName, CreationTime, EndTime, LastModifiedTime, FailureReason, PartialFailureReasons, BestCandidate, AutoMLJobStatus, AutoMLJobSecondaryStatus, AutoMLJobArtifacts, ResolvedAttributes, ModelDeployConfig, ModelDeployResult, DataSplitConfig, SecurityConfig, AutoMLComputeConfig. Pinned purpose: Returns information about an AutoML job created by calling CreateAutoMLJobV2 or CreateAutoMLJob .",
        ),
    ),
    (
        "DescribeCluster",
        Decision::Deferred(
            "Required selectors ClusterName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ClusterArn, ClusterName, ClusterStatus, CreationTime, FailureMessage, InstanceGroups, RestrictedInstanceGroups, RestrictedInstanceGroupsConfig, VpcConfig, Orchestrator, TieredStorageConfig, NodeRecovery, NodeProvisioningMode, ClusterRole, AutoScaling. Pinned purpose: Retrieves information of a SageMaker HyperPod cluster.",
        ),
    ),
    (
        "DescribeClusterEvent",
        Decision::Deferred(
            "Required selectors EventId, ClusterName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EventDetails. Pinned purpose: Retrieves detailed information about a specific event for a given HyperPod cluster. This functionality is only supported when the NodeProvisioningMode is set to Continuous .",
        ),
    ),
    (
        "DescribeClusterExtension",
        Decision::Deferred(
            "Required selectors ClusterArn, ExtensionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ExtensionDetails, NodeSummary, Health. Pinned purpose: Retrieves information about an extension installed on a SageMaker HyperPod cluster. The information includes the extension's version, configuration, and status, and a summary of how many cluster nodes are running the current version and configuration",
        ),
    ),
    (
        "DescribeClusterNode",
        Decision::Deferred(
            "Required selectors ClusterName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NodeDetails. Pinned purpose: Retrieves information of a node (also called a instance interchangeably) of a SageMaker HyperPod cluster.",
        ),
    ),
    (
        "DescribeClusterOperation",
        Decision::Deferred(
            "Required selectors ClusterName, OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OperationId, ClusterArn, Type, Status, CreationTime, ScalingDetails, DeepHealthCheckDetails. Pinned purpose: Retrieves detailed information about a specific operation for a given HyperPod cluster, such as a scaling operation or a deep health check. Use the OperationId returned by the ListClusterOperations operation to identify the operation to describe.",
        ),
    ),
    (
        "DescribeClusterSchedulerConfig",
        Decision::Deferred(
            "Required selectors ClusterSchedulerConfigId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ClusterSchedulerConfigArn, ClusterSchedulerConfigId, Name, ClusterSchedulerConfigVersion, Status, FailureReason, StatusDetails, ClusterArn, SchedulerConfig, Description, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy. Pinned purpose: Description of the cluster policy. This policy is used for task prioritization and fair-share allocation. This helps prioritize critical workloads and distributes idle compute across entities.",
        ),
    ),
];
