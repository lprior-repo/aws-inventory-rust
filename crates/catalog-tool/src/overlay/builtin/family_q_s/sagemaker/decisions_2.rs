// Pinned operation purposes: .local/aws-models/models/sagemaker/service/2017-07-24/sagemaker-2017-07-24.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Delete a hub.
    ("DeleteHub", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete the contents of a hub.
    ("DeleteHubContent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a hub content reference in order to remove a model from a private hub.
    (
        "DeleteHubContentReference",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to delete a human task user interface (worker task template). To see a list of human task user interfaces (work task templates) in your account, use ListHumanTaskUis . When you delete a worker task template, it no longer appears when you call ListHumanTaskUis .
    ("DeleteHumanTaskUi", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a hyperparameter tuning job. The DeleteHyperParameterTuningJob API deletes only the tuning job entry that was created in SageMaker when you called the CreateHyperParameterTuningJob API. It does not delete training jobs, artifacts, or the IAM role that you specified when creating the model.
    (
        "DeleteHyperParameterTuningJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a SageMaker AI image and all versions of the image. The container images aren't deleted.
    ("DeleteImage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a version of a SageMaker AI image. The container image the version represents isn't deleted.
    ("DeleteImageVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an inference component.
    (
        "DeleteInferenceComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an inference experiment. This operation does not delete your endpoint, variants, or any underlying resources. This operation only deletes the metadata of your experiment.
    (
        "DeleteInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a job. This operation is idempotent. If the job is currently running, you must stop it before deleting it by calling StopJob . The following operations are related to DeleteJob : CreateJob StopJob DescribeJob
    ("DeleteJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an MLflow App.
    ("DeleteMlflowApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an MLflow Tracking Server. For more information, see Clean up MLflow resources .
    (
        "DeleteMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a model. The DeleteModel API deletes only the model entry that was created in SageMaker when you called the CreateModel API. It does not delete model artifacts, inference code, or the IAM role that you specified when creating the model.
    ("DeleteModel", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon SageMaker AI model bias job definition.
    (
        "DeleteModelBiasJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon SageMaker Model Card.
    ("DeleteModelCard", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon SageMaker AI model explainability job definition.
    (
        "DeleteModelExplainabilityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a model package. A model package is used to create SageMaker models or list on Amazon Web Services Marketplace. Buyers can subscribe to model packages listed on Amazon Web Services Marketplace to create models in SageMaker.
    ("DeleteModelPackage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified model group.
    (
        "DeleteModelPackageGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a model group resource policy.
    (
        "DeleteModelPackageGroupPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the secified model quality monitoring job definition.
    (
        "DeleteModelQualityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a monitoring schedule. Also stops the schedule had not already been stopped. This does not delete the job execution history of the monitoring schedule.
    (
        "DeleteMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an SageMaker AI notebook instance. Before you can delete a notebook instance, you must call the StopNotebookInstance API. When you delete a notebook instance, you lose all of your data. SageMaker AI removes the ML compute instance, and deletes the ML storage volume and the network interface associated with the notebook instance.
    (
        "DeleteNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a notebook instance lifecycle configuration.
    (
        "DeleteNotebookInstanceLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an optimization job.
    (
        "DeleteOptimizationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a SageMaker Partner AI App.
    ("DeletePartnerApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a pipeline if there are no running instances of the pipeline. To delete a pipeline, you must stop all running instances of the pipeline using the StopPipelineExecution API. When you delete a pipeline, all instances of the pipeline are deleted.
    ("DeletePipeline", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a processing job. After Amazon SageMaker deletes a processing job, all of the metadata for the processing job is lost. You can delete only processing jobs that are in a terminal state ( Stopped , Failed , or Completed ). You cannot delete a job that is in the InProgress or Stopping state. After deleting the job, you can reuse its name to create another processing job.
    (
        "DeleteProcessingJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Delete the specified project.
    ("DeleteProject", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Used to delete a space.
    ("DeleteSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the Amazon SageMaker AI Studio Lifecycle Configuration. In order to delete the Lifecycle Configuration, there must be no running apps using the Lifecycle Configuration. You must also remove the Lifecycle Configuration from UserSettings in all Domains and UserProfiles.
    (
        "DeleteStudioLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified tags from an SageMaker resource. To list a resource's tags, use the ListTags API. When you call this API to delete tags from a hyperparameter tuning job, the deleted tags are not removed from training jobs that the hyperparameter tuning job launched before you called this API. When you call this API to delete tags from a SageMaker Domain or User Profile, the deleted tags are
    ("DeleteTags", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a training job. After SageMaker deletes a training job, all of the metadata for the training job is lost. You can delete only training jobs that are in a terminal state ( Stopped , Failed , or Completed ) and don't retain an Available managed warm pool . You cannot delete a job that is in the InProgress or Stopping state. After deleting the job, you can reuse its name to create another tra
    ("DeleteTrainingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a training session. You must stop a running session before deleting it. Use the StopTrainingSession operation to stop the session first. The following operations are related to DeleteTrainingSession : CreateTrainingSession DescribeTrainingSession StopTrainingSession
    (
        "DeleteTrainingSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified trial. All trial components that make up the trial must be deleted first. Use the DescribeTrialComponent API to get the list of trial components.
    ("DeleteTrial", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified trial component. A trial component must be disassociated from all trials before the trial component can be deleted. To disassociate a trial component from a trial, call the DisassociateTrialComponent API.
    (
        "DeleteTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a user profile. When a user profile is deleted, the user loses access to their EFS volume, including data, notebooks, and other artifacts.
    ("DeleteUserProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to delete a workforce. If you want to create a new workforce in an Amazon Web Services Region where a workforce already exists, use this operation to delete the existing workforce and then use CreateWorkforce to create a new workforce. If a private workforce contains one or more work teams, you must use the DeleteWorkteam operation to delete all work teams before you delete the
    ("DeleteWorkforce", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an existing work team. This operation can't be undone.
    ("DeleteWorkteam", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deregisters the specified devices. After you deregister a device, you will need to re-register the devices.
    ("DeregisterDevices", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeAIBenchmarkJob",
        Decision::Deferred(
            "Required selectors AIBenchmarkJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AIBenchmarkJobName, AIBenchmarkJobArn, AIBenchmarkJobStatus, FailureReason, BenchmarkTarget, OutputConfig, AIWorkloadConfigIdentifier, RoleArn, NetworkConfig, CreationTime, StartTime, EndTime, Tags. Pinned purpose: Returns details of an AI benchmark job, including its status, configuration, target endpoint, and timing information.",
        ),
    ),
    // Full modeled response contains authentication/secret material Recommendations[].DeploymentConfiguration.EnvironmentVariables. Pinned purpose: Returns details of an AI recommendation job, including its status, model source, performance targets, optimization recommendations, and deployment configurations.. Projection cannot authorize this call.
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
