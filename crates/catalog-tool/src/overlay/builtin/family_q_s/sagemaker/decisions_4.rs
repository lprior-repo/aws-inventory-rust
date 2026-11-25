use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DescribeTrainingSession",
        Decision::Deferred(
            "Required selectors TrainingSessionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrainingSessionDetail. Pinned purpose: Retrieves detailed information about a training session, including its current status, model configuration, Low-Rank Adaptation (LoRA) parameters, and timestamps. The following operations are related to DescribeTrainingSession : CreateTrainingSession",
        ),
    ),
    (
        "DescribeTransformJob",
        Decision::Deferred(
            "Required selectors TransformJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TransformJobName, TransformJobArn, TransformJobStatus, FailureReason, ModelName, MaxConcurrentTransforms, ModelClientConfig, MaxPayloadInMB, BatchStrategy, Environment, TransformInput, TransformOutput, DataCaptureConfig, TransformResources, CreationTime, TransformStartTime, TransformEndTime, LabelingJobArn, AutoMLJobArn, DataProcessing, ExperimentConfig. Pinned purpose: Returns information about a transform job.",
        ),
    ),
    (
        "DescribeTrial",
        Decision::Deferred(
            "Required selectors TrialName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrialName, TrialArn, DisplayName, ExperimentName, Source, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, MetadataProperties. Pinned purpose: Provides a list of a trial's properties.",
        ),
    ),
    (
        "DescribeTrialComponent",
        Decision::Deferred(
            "Required selectors TrialComponentName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrialComponentName, TrialComponentArn, DisplayName, Source, Status, StartTime, EndTime, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, Parameters, InputArtifacts, OutputArtifacts, MetadataProperties, Metrics, LineageGroupArn, Sources. Pinned purpose: Provides a list of a trials component's properties.",
        ),
    ),
    (
        "DescribeUserProfile",
        Decision::Deferred(
            "Required selectors DomainId, UserProfileName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainId, UserProfileArn, UserProfileName, HomeEfsFileSystemUid, Status, LastModifiedTime, CreationTime, FailureReason, SingleSignOnUserIdentifier, SingleSignOnUserValue, UserSettings. Pinned purpose: Describes a user profile. For more information, see CreateUserProfile .",
        ),
    ),
    (
        "DescribeWorkforce",
        Decision::Deferred(
            "Required selectors WorkforceName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Workforce. Pinned purpose: Lists private workforce information, including workforce name, Amazon Resource Name (ARN), and, if applicable, allowed IP address ranges ( CIDRs ). Allowable IP address ranges are the IP addresses that workers can use to access tasks. This operation ",
        ),
    ),
    (
        "DescribeWorkteam",
        Decision::Deferred(
            "Required selectors WorkteamName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Workteam. Pinned purpose: Gets information about a specific work team. You can see information such as the creation date, the last updated date, membership information, and the work team's Amazon Resource Name (ARN).",
        ),
    ),
    (
        "DetachClusterNodeVolume",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisableSagemakerServicecatalogPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableSagemakerServicecatalogPortfolio",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExtendTrainingPlan",
        Decision::Deferred(
            "Required selectors TrainingPlanExtensionOfferingId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrainingPlanExtensions. Pinned purpose: Extends an existing training plan by purchasing an extension offering. This allows you to add additional compute capacity time to your training plan without creating a new plan or reconfiguring your workloads. To find available extension offerings, u",
        ),
    ),
    (
        "GetDeviceFleetReport",
        Decision::Deferred(
            "Required selectors DeviceFleetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DeviceFleetArn, DeviceFleetName, OutputConfig, Description, ReportGenerated, DeviceStats, AgentVersions, ModelStats. Pinned purpose: Describes a fleet.",
        ),
    ),
    (
        "GetLineageGroupPolicy",
        Decision::Deferred(
            "Required selectors LineageGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LineageGroupArn, ResourcePolicy. Pinned purpose: The resource policy for the lineage group.",
        ),
    ),
    (
        "GetModelPackageGroupPolicy",
        Decision::Deferred(
            "Required selectors ModelPackageGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourcePolicy. Pinned purpose: Gets a resource policy that manages access for a model group. For information about resource policies, see Identity-based policies and resource-based policies in the Amazon Web Services Identity and Access Management User Guide. .",
        ),
    ),
    (
        "GetSagemakerServicecatalogPortfolioStatus",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Status. Pinned request/ownership contract: Gets the status of Service Catalog in SageMaker. Service Catalog is used to create SageMaker projects.",
        ),
    ),
    (
        "GetScalingConfigurationRecommendation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetSearchSuggestions",
        Decision::Deferred(
            "Required selectors Resource are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PropertyNameSuggestions. Pinned purpose: An auto-complete API for the search functionality in the SageMaker console. It returns suggestions of possible matches for the property name to use in Search queries. Provides suggestions for HyperParameters , Tags , and Metrics .",
        ),
    ),
    ("ImportHubContent", Decision::Denied(DenyReason::Mutating)),
    ("ListAppImageConfigs", Decision::Denied(DenyReason::Secret)),
    (
        "ListClusterExtensionVersions",
        Decision::Deferred(
            "Required selectors ExtensionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ExtensionVersions, NextToken. Pinned purpose: Lists the versions of an extension that you can install on a SageMaker HyperPod cluster, from the highest version to the lowest. Use a version from this list as the ExtensionVersion in CreateClusterExtension or UpdateClusterExtension . The list conta",
        ),
    ),
    (
        "ListClusterNodes",
        Decision::Deferred(
            "Required selectors ClusterName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, ClusterNodeSummaries. Pinned purpose: Retrieves the list of instances (also called nodes interchangeably) in a SageMaker HyperPod cluster.",
        ),
    ),
    (
        "ListClusterOperations",
        Decision::Deferred(
            "Required selectors ClusterName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, ClusterOperationSummaries. Pinned purpose: Retrieves a list of operation summaries for a specified HyperPod cluster, such as scaling operations and deep health checks. The operation supports filtering, sorting, and pagination of results.",
        ),
    ),
    (
        "ListHubContentVersions",
        Decision::Deferred(
            "Required selectors HubName, HubContentType, HubContentName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HubContentSummaries, NextToken. Pinned purpose: List hub content versions.",
        ),
    ),
    (
        "ListHubContents",
        Decision::Deferred(
            "Required selectors HubName, HubContentType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HubContentSummaries, NextToken. Pinned purpose: List the contents of a hub.",
        ),
    ),
    (
        "ListInferenceRecommendationsJobSteps",
        Decision::Deferred(
            "Required selectors JobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Steps, NextToken. Pinned purpose: Returns a list of the subtasks for an Inference Recommender job. The supported subtasks are benchmarks, which evaluate the performance of your model on different instance types.",
        ),
    ),
    (
        "ListJobSchemaVersions",
        Decision::Deferred(
            "Required selectors JobCategory are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, JobConfigSchemas. Pinned purpose: Lists available configuration schema versions for a specified job category. Use the schema versions with DescribeJobSchemaVersion to retrieve the full schema document. The following operations are related to ListJobSchemaVersions : DescribeJobSchemaV",
        ),
    ),
    (
        "ListJobs",
        Decision::Deferred(
            "Required selectors JobCategory are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, JobSummaries. Pinned purpose: Lists jobs in a specified category. You can filter results by creation time, last modified time, name, and status. Results are sorted by the field you specify in SortBy . Use pagination to retrieve large result sets efficiently. The following operati",
        ),
    ),
    (
        "ListLabelingJobsForWorkteam",
        Decision::Deferred(
            "Required selectors WorkteamArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LabelingJobSummaryList, NextToken. Pinned purpose: Gets a list of labeling jobs assigned to a specified work team.",
        ),
    ),
    (
        "ListLineageGroups",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListModelCardExportJobs",
        Decision::Deferred(
            "Required selectors ModelCardName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelCardExportJobSummaries, NextToken. Pinned purpose: List the export jobs for the Amazon SageMaker Model Card.",
        ),
    ),
    (
        "ListModelCardVersions",
        Decision::Deferred(
            "Required selectors ModelCardName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelCardVersionSummaryList, NextToken. Pinned purpose: List existing versions of an Amazon SageMaker Model Card.",
        ),
    ),
    (
        "ListModelMetadata",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListMonitoringAlertHistory",
        Decision::Deferred(
            "Returns alert history observations rather than independently identified configuration resources; event identity needs a documented schedule/alert/timestamp compound contract.",
        ),
    ),
    (
        "ListMonitoringAlerts",
        Decision::Deferred(
            "Required selectors MonitoringScheduleName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MonitoringAlertSummaries, NextToken. Pinned purpose: Gets the alerts for a single monitoring schedule.",
        ),
    ),
    (
        "ListMonitoringExecutions",
        Decision::Deferred(
            "Summary identity depends on schedule name and execution ScheduledTime; ProcessingJobArn is absent before a job starts. Requires an explicit timestamp-stable execution identity rather than treating an optional processing job ARN as universal.",
        ),
    ),
    (
        "ListPipelineExecutionSteps",
        Decision::Deferred(
            "Requires a PipelineExecutionArn selector semantically even though optional in the pinned input. Returns execution step metadata, not complete pipeline root inventory.",
        ),
    ),
    (
        "ListPipelineExecutions",
        Decision::Deferred(
            "Required selectors PipelineName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineExecutionSummaries, NextToken. Pinned purpose: Gets a list of the pipeline executions.",
        ),
    ),
    (
        "ListPipelineParametersForExecution",
        Decision::Deferred(
            "Required selectors PipelineExecutionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineParameters, NextToken. Pinned purpose: Gets a list of parameters for a pipeline execution.",
        ),
    ),
    (
        "ListPipelineVersions",
        Decision::Deferred(
            "Required selectors PipelineName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineVersionSummaries, NextToken. Pinned purpose: Gets a list of all versions of the pipeline.",
        ),
    ),
    (
        "ListStageDevices",
        Decision::Deferred(
            "Required selectors EdgeDeploymentPlanName, StageName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DeviceDeploymentSummaries, NextToken. Pinned purpose: Lists devices allocated to the stage, containing detailed device information and deployment status.",
        ),
    ),
    (
        "ListSubscribedWorkteams",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTags",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Returns the tags for the specified SageMaker resource.",
        ),
    ),
    (
        "ListTrainingJobsForHyperParameterTuningJob",
        Decision::Deferred(
            "Required selectors HyperParameterTuningJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrainingJobSummaries, NextToken. Pinned purpose: Gets a list of TrainingJobSummary objects that describe the training jobs that a hyperparameter tuning job launched.",
        ),
    ),
    (
        "ListUltraServersByReservedCapacity",
        Decision::Deferred(
            "Required selectors ReservedCapacityArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, UltraServers. Pinned purpose: Lists all UltraServers that are part of a specified reserved capacity.",
        ),
    ),
    (
        "PutModelPackageGroupPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "QueryLineage",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Vertices, Edges, NextToken. Pinned request/ownership contract: Use this action to inspect your lineage and discover relationships between entities. For more information, see Querying Lineage Entities in the Amazon SageMaker Developer Guide .",
        ),
    ),
    ("RegisterDevices", Decision::Denied(DenyReason::Mutating)),
];
