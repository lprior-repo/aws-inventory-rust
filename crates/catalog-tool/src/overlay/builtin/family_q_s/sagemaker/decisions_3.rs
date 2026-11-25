use super::super::construction::Decision;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DescribeCodeRepository",
        Decision::Deferred(
            "Required selectors CodeRepositoryName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CodeRepositoryName, CodeRepositoryArn, CreationTime, LastModifiedTime, GitConfig. Pinned purpose: Gets details about the specified Git repository.",
        ),
    ),
    (
        "DescribeCompilationJob",
        Decision::Deferred(
            "Required selectors CompilationJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CompilationJobName, CompilationJobArn, CompilationJobStatus, CompilationStartTime, CompilationEndTime, StoppingCondition, InferenceImage, ModelPackageVersionArn, CreationTime, LastModifiedTime, FailureReason, ModelArtifacts, ModelDigests, RoleArn, InputConfig, OutputConfig, VpcConfig, DerivedInformation. Pinned purpose: Returns information about a model compilation job. To create a model compilation job, use CreateCompilationJob . To get information about multiple model compilation jobs, use ListCompilationJobs .",
        ),
    ),
    (
        "DescribeComputeQuota",
        Decision::Deferred(
            "Required selectors ComputeQuotaId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ComputeQuotaArn, ComputeQuotaId, Name, Description, ComputeQuotaVersion, Status, FailureReason, ClusterArn, ComputeQuotaConfig, ComputeQuotaTarget, ActivationState, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy. Pinned purpose: Description of the compute allocation definition.",
        ),
    ),
    (
        "DescribeContext",
        Decision::Deferred(
            "Required selectors ContextName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ContextName, ContextArn, Source, ContextType, Description, Properties, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, LineageGroupArn. Pinned purpose: Describes a context.",
        ),
    ),
    (
        "DescribeDataQualityJobDefinition",
        Decision::Deferred(
            "Required selectors JobDefinitionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobDefinitionArn, JobDefinitionName, CreationTime, DataQualityBaselineConfig, DataQualityAppSpecification, DataQualityJobInput, DataQualityJobOutputConfig, JobResources, NetworkConfig, RoleArn, StoppingCondition. Pinned purpose: Gets the details of a data quality monitoring job definition.",
        ),
    ),
    (
        "DescribeDevice",
        Decision::Deferred(
            "Required selectors DeviceName, DeviceFleetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DeviceArn, DeviceName, Description, DeviceFleetName, IotThingName, RegistrationTime, LatestHeartbeat, Models, MaxModels, NextToken, AgentVersion. Pinned purpose: Describes the device.",
        ),
    ),
    (
        "DescribeDeviceFleet",
        Decision::Deferred(
            "Required selectors DeviceFleetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DeviceFleetName, DeviceFleetArn, OutputConfig, Description, CreationTime, LastModifiedTime, RoleArn, IotRoleAlias. Pinned purpose: A description of the fleet the device belongs to.",
        ),
    ),
    (
        "DescribeDomain",
        Decision::Deferred(
            "Required selectors DomainId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainArn, DomainId, DomainName, HomeEfsFileSystemId, SingleSignOnManagedApplicationInstanceId, SingleSignOnApplicationArn, Status, CreationTime, LastModifiedTime, FailureReason, SecurityGroupIdForDomainBoundary, AuthMode, DefaultUserSettings, DomainSettings, AppNetworkAccessType, HomeEfsFileSystemKmsKeyId, SubnetIds, Url, VpcId, KmsKeyId, AppSecurityGroupManagement, HomeEfsFileSystemCreation, TagPropagation, DefaultSpaceSettings. Pinned purpose: The description of the domain.",
        ),
    ),
    (
        "DescribeEdgeDeploymentPlan",
        Decision::Deferred(
            "Required selectors EdgeDeploymentPlanName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EdgeDeploymentPlanArn, EdgeDeploymentPlanName, ModelConfigs, DeviceFleetName, EdgeDeploymentSuccess, EdgeDeploymentPending, EdgeDeploymentFailed, Stages, NextToken, CreationTime, LastModifiedTime. Pinned purpose: Describes an edge deployment plan with deployment status per stage.",
        ),
    ),
    (
        "DescribeEdgePackagingJob",
        Decision::Deferred(
            "Required selectors EdgePackagingJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EdgePackagingJobArn, EdgePackagingJobName, CompilationJobName, ModelName, ModelVersion, RoleArn, OutputConfig, ResourceKey, EdgePackagingJobStatus, EdgePackagingJobStatusMessage, CreationTime, LastModifiedTime, ModelArtifact, ModelSignature, PresetDeploymentOutput. Pinned purpose: A description of edge packaging jobs.",
        ),
    ),
    (
        "DescribeEndpoint",
        Decision::Deferred(
            "Required selectors EndpointName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EndpointName, EndpointArn, EndpointConfigName, ProductionVariants, DataCaptureConfig, EndpointStatus, FailureReason, CreationTime, LastModifiedTime, LastDeploymentConfig, AsyncInferenceConfig, PendingDeploymentSummary, ExplainerConfig, ShadowProductionVariants, MetricsConfig. Pinned purpose: Returns the description of an endpoint.",
        ),
    ),
    (
        "DescribeEndpointConfig",
        Decision::Deferred(
            "Required selectors EndpointConfigName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EndpointConfigName, EndpointConfigArn, ProductionVariants, DataCaptureConfig, KmsKeyId, CreationTime, AsyncInferenceConfig, ExplainerConfig, ShadowProductionVariants, ExecutionRoleArn, VpcConfig, EnableNetworkIsolation, MetricsConfig. Pinned purpose: Returns the description of an endpoint configuration created using the CreateEndpointConfig API.",
        ),
    ),
    (
        "DescribeExperiment",
        Decision::Deferred(
            "Required selectors ExperimentName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ExperimentName, ExperimentArn, DisplayName, Source, Description, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy. Pinned purpose: Provides a list of an experiment's properties.",
        ),
    ),
    (
        "DescribeFeatureGroup",
        Decision::Deferred(
            "Required selectors FeatureGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FeatureGroupArn, FeatureGroupName, RecordIdentifierFeatureName, EventTimeFeatureName, FeatureDefinitions, CreationTime, LastModifiedTime, OnlineStoreConfig, OfflineStoreConfig, ThroughputConfig, RoleArn, FeatureGroupStatus, OfflineStoreStatus, LastUpdateStatus, FailureReason, Description, NextToken, OnlineStoreTotalSizeBytes. Pinned purpose: Use this operation to describe a FeatureGroup . The response includes information on the creation time, FeatureGroup name, the unique identifier for each FeatureGroup , and more.",
        ),
    ),
    (
        "DescribeFeatureMetadata",
        Decision::Deferred(
            "Required selectors FeatureGroupName, FeatureName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FeatureGroupArn, FeatureGroupName, FeatureName, FeatureType, CreationTime, LastModifiedTime, Description, Parameters. Pinned purpose: Shows the metadata for a feature within a feature group.",
        ),
    ),
    (
        "DescribeFlowDefinition",
        Decision::Deferred(
            "Required selectors FlowDefinitionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FlowDefinitionArn, FlowDefinitionName, FlowDefinitionStatus, CreationTime, HumanLoopRequestSource, HumanLoopActivationConfig, HumanLoopConfig, OutputConfig, RoleArn, FailureReason. Pinned purpose: Returns information about the specified flow definition.",
        ),
    ),
    (
        "DescribeHub",
        Decision::Deferred(
            "Required selectors HubName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HubName, HubArn, HubDisplayName, HubDescription, HubSearchKeywords, S3StorageConfig, HubStatus, FailureReason, CreationTime, LastModifiedTime, CreatedBy, LastModifiedBy. Pinned purpose: Describes a hub.",
        ),
    ),
    (
        "DescribeHubContent",
        Decision::Deferred(
            "Required selectors HubName, HubContentType, HubContentName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HubContentName, HubContentArn, HubContentVersion, HubContentType, DocumentSchemaVersion, HubName, HubArn, HubContentDisplayName, HubContentDescription, HubContentMarkdown, HubContentDocument, SageMakerPublicHubContentArn, ReferenceMinVersion, SupportStatus, HubContentSearchKeywords, HubContentDependencies, HubContentStatus, FailureReason, CreationTime, LastModifiedTime, CreatedBy, LastModifiedBy. Pinned purpose: Describe the content of a hub.",
        ),
    ),
    (
        "DescribeHumanTaskUi",
        Decision::Deferred(
            "Required selectors HumanTaskUiName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HumanTaskUiArn, HumanTaskUiName, HumanTaskUiStatus, CreationTime, UiTemplate. Pinned purpose: Returns information about the requested human task user interface (worker task template).",
        ),
    ),
    (
        "DescribeHyperParameterTuningJob",
        Decision::Deferred(
            "Required selectors HyperParameterTuningJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members HyperParameterTuningJobName, HyperParameterTuningJobArn, HyperParameterTuningJobConfig, TrainingJobDefinition, TrainingJobDefinitions, HyperParameterTuningJobStatus, CreationTime, HyperParameterTuningEndTime, LastModifiedTime, TrainingJobStatusCounters, ObjectiveStatusCounters, BestTrainingJob, OverallBestTrainingJob, WarmStartConfig, Autotune, FailureReason, TuningJobCompletionDetails, ConsumedResources. Pinned purpose: Returns a description of a hyperparameter tuning job, depending on the fields selected. These fields can include the name, Amazon Resource Name (ARN), job status of your tuning job and more.",
        ),
    ),
    (
        "DescribeImage",
        Decision::Deferred(
            "Required selectors ImageName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CreationTime, Description, DisplayName, FailureReason, ImageArn, ImageName, ImageStatus, LastModifiedTime, RoleArn. Pinned purpose: Describes a SageMaker AI image.",
        ),
    ),
    (
        "DescribeImageVersion",
        Decision::Deferred(
            "Required selectors ImageName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members BaseImage, ContainerImage, CreationTime, FailureReason, ImageArn, ImageVersionArn, ImageVersionStatus, LastModifiedTime, Version, VendorGuidance, JobType, MLFramework, ProgrammingLang, Processor, Horovod, ReleaseNotes. Pinned purpose: Describes a version of a SageMaker AI image.",
        ),
    ),
    (
        "DescribeInferenceComponent",
        Decision::Deferred(
            "Required selectors InferenceComponentName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InferenceComponentName, InferenceComponentArn, EndpointName, EndpointArn, VariantName, FailureReason, Specification, Specifications, RuntimeConfig, CreationTime, LastModifiedTime, InferenceComponentStatus, LastDeploymentConfig. Pinned purpose: Returns information about an inference component.",
        ),
    ),
    (
        "DescribeInferenceExperiment",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Name, Type, Schedule, Status, StatusReason, Description, CreationTime, CompletionTime, LastModifiedTime, RoleArn, EndpointMetadata, ModelVariants, DataStorageConfig, ShadowModeConfig, KmsKey. Pinned purpose: Returns details about an inference experiment.",
        ),
    ),
    (
        "DescribeInferenceRecommendationsJob",
        Decision::Deferred(
            "Required selectors JobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobName, JobDescription, JobType, JobArn, RoleArn, Status, CreationTime, CompletionTime, LastModifiedTime, FailureReason, InputConfig, StoppingConditions, InferenceRecommendations, EndpointPerformances. Pinned purpose: Provides the results of the Inference Recommender job. One or more recommendation jobs are returned.",
        ),
    ),
    (
        "DescribeJob",
        Decision::Deferred(
            "Required selectors JobName, JobCategory are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobName, JobArn, RoleArn, JobCategory, JobConfigSchemaVersion, JobConfigDocument, CreationTime, LastModifiedTime, EndTime, JobStatus, SecondaryStatus, SecondaryStatusTransitions, FailureReason, Tags. Pinned purpose: Returns detailed information about a job, including its current status, secondary status, configuration, and timestamps. Use SecondaryStatus for granular progress tracking and SecondaryStatusTransitions to see the full history of status changes with ",
        ),
    ),
    (
        "DescribeJobSchemaVersion",
        Decision::Deferred(
            "Required selectors JobCategory are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobCategory, JobConfigSchemaVersion, JobConfigSchema. Pinned purpose: Returns the JSON schema for a specified job category and schema version. Use this schema to validate your JobConfigDocument before calling CreateJob . If you don't specify a schema version, the latest version is returned. The schema defines required ",
        ),
    ),
    (
        "DescribeLabelingJob",
        Decision::Deferred(
            "Required selectors LabelingJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LabelingJobStatus, LabelCounters, FailureReason, CreationTime, LastModifiedTime, JobReferenceCode, LabelingJobName, LabelingJobArn, LabelAttributeName, InputConfig, OutputConfig, RoleArn, LabelCategoryConfigS3Uri, StoppingConditions, LabelingJobAlgorithmsConfig, HumanTaskConfig, Tags, LabelingJobOutput. Pinned purpose: Gets information about a labeling job.",
        ),
    ),
    (
        "DescribeLineageGroup",
        Decision::Deferred(
            "Required selectors LineageGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LineageGroupName, LineageGroupArn, DisplayName, Description, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy. Pinned purpose: Provides a list of properties for the requested lineage group. For more information, see Cross-Account Lineage Tracking in the Amazon SageMaker Developer Guide .",
        ),
    ),
    (
        "DescribeMlflowApp",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Name, ArtifactStoreUri, MlflowVersion, RoleArn, KmsKeyId, Status, ModelRegistrationMode, AccountDefaultStatus, DefaultDomainIdList, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, WeeklyMaintenanceWindowStart, MaintenanceStatus. Pinned purpose: Returns information about an MLflow App.",
        ),
    ),
    (
        "DescribeMlflowTrackingServer",
        Decision::Deferred(
            "Required selectors TrackingServerName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrackingServerArn, TrackingServerName, ArtifactStoreUri, TrackingServerSize, MlflowVersion, RoleArn, TrackingServerStatus, TrackingServerMaintenanceStatus, IsActive, TrackingServerUrl, WeeklyMaintenanceWindowStart, AutomaticModelRegistration, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, S3BucketOwnerAccountId, S3BucketOwnerVerification. Pinned purpose: Returns information about an MLflow Tracking Server.",
        ),
    ),
    (
        "DescribeModel",
        Decision::Deferred(
            "Required selectors ModelName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelName, PrimaryContainer, Containers, InferenceExecutionConfig, ExecutionRoleArn, VpcConfig, CreationTime, ModelArn, EnableNetworkIsolation, DeploymentRecommendation. Pinned purpose: Describes a model that you created using the CreateModel API.",
        ),
    ),
    (
        "DescribeModelBiasJobDefinition",
        Decision::Deferred(
            "Required selectors JobDefinitionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobDefinitionArn, JobDefinitionName, CreationTime, ModelBiasBaselineConfig, ModelBiasAppSpecification, ModelBiasJobInput, ModelBiasJobOutputConfig, JobResources, NetworkConfig, RoleArn, StoppingCondition. Pinned purpose: Returns a description of a model bias job definition.",
        ),
    ),
    (
        "DescribeModelCard",
        Decision::Deferred(
            "Required selectors ModelCardName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelCardArn, ModelCardName, ModelCardVersion, Content, ModelCardStatus, SecurityConfig, CreationTime, CreatedBy, LastModifiedTime, LastModifiedBy, ModelCardProcessingStatus. Pinned purpose: Describes the content, creation time, and security configuration of an Amazon SageMaker Model Card. To retrieve only metadata about a model card without requiring kms:Decrypt permission on the associated customer-managed Amazon Web Services KMS key, ",
        ),
    ),
    (
        "DescribeModelCardExportJob",
        Decision::Deferred(
            "Required selectors ModelCardExportJobArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelCardExportJobName, ModelCardExportJobArn, Status, ModelCardName, ModelCardVersion, OutputConfig, CreatedAt, LastModifiedAt, FailureReason, ExportArtifacts. Pinned purpose: Describes an Amazon SageMaker Model Card export job.",
        ),
    ),
    (
        "DescribeModelExplainabilityJobDefinition",
        Decision::Deferred(
            "Required selectors JobDefinitionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobDefinitionArn, JobDefinitionName, CreationTime, ModelExplainabilityBaselineConfig, ModelExplainabilityAppSpecification, ModelExplainabilityJobInput, ModelExplainabilityJobOutputConfig, JobResources, NetworkConfig, RoleArn, StoppingCondition. Pinned purpose: Returns a description of a model explainability job definition.",
        ),
    ),
    (
        "DescribeModelPackage",
        Decision::Deferred(
            "Required selectors ModelPackageName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelPackageName, ModelPackageGroupName, ModelPackageVersion, ModelPackageRegistrationType, ModelPackageArn, ModelPackageDescription, CreationTime, InferenceSpecification, SourceAlgorithmSpecification, ValidationSpecification, ModelPackageStatus, ModelPackageStatusDetails, CertifyForMarketplace, ModelApprovalStatus, CreatedBy, MetadataProperties, ModelMetrics, LastModifiedTime, LastModifiedBy, ApprovalDescription, Domain, Task, SamplePayloadUrl, CustomerMetadataProperties, DriftCheckBaselines, AdditionalInferenceSpecifications, SkipModelValidation, SourceUri, SecurityConfig, ModelCard, ModelLifeCycle, ManagedStorageType. Pinned purpose: Returns a description of the specified model package, which is used to create SageMaker models or list them on Amazon Web Services Marketplace. If you provided a KMS Key ID when you created your model package, you will see the KMS Decrypt API call in",
        ),
    ),
    (
        "DescribeModelPackageGroup",
        Decision::Deferred(
            "Required selectors ModelPackageGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ModelPackageGroupName, ModelPackageGroupArn, ModelPackageGroupDescription, CreationTime, CreatedBy, ModelPackageGroupStatus, ManagedConfiguration. Pinned purpose: Gets a description for the specified model group.",
        ),
    ),
    (
        "DescribeModelQualityJobDefinition",
        Decision::Deferred(
            "Required selectors JobDefinitionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobDefinitionArn, JobDefinitionName, CreationTime, ModelQualityBaselineConfig, ModelQualityAppSpecification, ModelQualityJobInput, ModelQualityJobOutputConfig, JobResources, NetworkConfig, RoleArn, StoppingCondition. Pinned purpose: Returns a description of a model quality job definition.",
        ),
    ),
    (
        "DescribeMonitoringSchedule",
        Decision::Deferred(
            "Required selectors MonitoringScheduleName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MonitoringScheduleArn, MonitoringScheduleName, MonitoringScheduleStatus, MonitoringType, FailureReason, CreationTime, LastModifiedTime, MonitoringScheduleConfig, EndpointName, LastMonitoringExecutionSummary. Pinned purpose: Describes the schedule for a monitoring job.",
        ),
    ),
    (
        "DescribeNotebookInstance",
        Decision::Deferred(
            "Required selectors NotebookInstanceName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NotebookInstanceArn, NotebookInstanceName, NotebookInstanceStatus, FailureReason, Url, InstanceType, IpAddressType, SubnetId, SecurityGroups, RoleArn, KmsKeyId, NetworkInterfaceId, LastModifiedTime, CreationTime, NotebookInstanceLifecycleConfigName, DirectInternetAccess, VolumeSizeInGB, AcceleratorTypes, DefaultCodeRepository, AdditionalCodeRepositories, RootAccess, PlatformIdentifier, InstanceMetadataServiceConfiguration. Pinned purpose: Returns information about a notebook instance.",
        ),
    ),
    (
        "DescribeNotebookInstanceLifecycleConfig",
        Decision::Deferred(
            "Required selectors NotebookInstanceLifecycleConfigName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NotebookInstanceLifecycleConfigArn, NotebookInstanceLifecycleConfigName, OnCreate, OnStart, LastModifiedTime, CreationTime. Pinned purpose: Returns a description of a notebook instance lifecycle configuration. For information about notebook instance lifestyle configurations, see Step 2.1: (Optional) Customize a Notebook Instance .",
        ),
    ),
    (
        "DescribeOptimizationJob",
        Decision::Deferred(
            "Required selectors OptimizationJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OptimizationJobArn, OptimizationJobStatus, OptimizationStartTime, OptimizationEndTime, CreationTime, LastModifiedTime, FailureReason, OptimizationJobName, ModelSource, OptimizationEnvironment, DeploymentInstanceType, MaxInstanceCount, OptimizationConfigs, OutputConfig, OptimizationOutput, RoleArn, StoppingCondition, VpcConfig, TrainingPlanArns. Pinned purpose: Provides the properties of the specified optimization job.",
        ),
    ),
    (
        "DescribePartnerApp",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Name, Type, Status, CreationTime, LastModifiedTime, ExecutionRoleArn, KmsKeyId, BaseUrl, MaintenanceConfig, Tier, Version, ApplicationConfig, AuthType, EnableIamSessionBasedIdentity, Error, EnableAutoMinorVersionUpgrade, CurrentVersionEolDate, AvailableUpgrade, IdcConfig. Pinned purpose: Gets information about a SageMaker Partner AI App.",
        ),
    ),
    (
        "DescribePipeline",
        Decision::Deferred(
            "Required selectors PipelineName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineArn, PipelineName, PipelineDisplayName, PipelineDefinition, PipelineDescription, RoleArn, PipelineStatus, CreationTime, LastModifiedTime, LastRunTime, CreatedBy, LastModifiedBy, ParallelismConfiguration, PipelineVersionDisplayName, PipelineVersionDescription, PipelineVersionAliases. Pinned purpose: Describes the details of a pipeline.",
        ),
    ),
    (
        "DescribePipelineDefinitionForExecution",
        Decision::Deferred(
            "Required selectors PipelineExecutionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineDefinition, CreationTime. Pinned purpose: Describes the details of an execution's pipeline definition.",
        ),
    ),
    (
        "DescribePipelineExecution",
        Decision::Deferred(
            "Required selectors PipelineExecutionArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PipelineArn, PipelineExecutionArn, PipelineExecutionDisplayName, PipelineExecutionStatus, PipelineExecutionDescription, PipelineExperimentConfig, FailureReason, CreationTime, LastModifiedTime, CreatedBy, LastModifiedBy, ParallelismConfiguration, SelectiveExecutionConfig, PipelineVersionId, MLflowConfig, PipelineVersionAliases, RoleArn. Pinned purpose: Describes the details of a pipeline execution.",
        ),
    ),
    (
        "DescribeProcessingJob",
        Decision::Deferred(
            "Required selectors ProcessingJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProcessingInputs, ProcessingOutputConfig, ProcessingJobName, ProcessingResources, StoppingCondition, AppSpecification, Environment, NetworkConfig, RoleArn, ExperimentConfig, ProcessingJobArn, ProcessingJobStatus, ExitMessage, FailureReason, ProcessingEndTime, ProcessingStartTime, LastModifiedTime, CreationTime, MonitoringScheduleArn, AutoMLJobArn, TrainingJobArn. Pinned purpose: Returns a description of a processing job.",
        ),
    ),
    (
        "DescribeProject",
        Decision::Deferred(
            "Required selectors ProjectName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProjectArn, ProjectName, ProjectId, ProjectDescription, ServiceCatalogProvisioningDetails, ServiceCatalogProvisionedProductDetails, ProjectStatus, TemplateProviderDetails, CreatedBy, CreationTime, LastModifiedTime, LastModifiedBy. Pinned purpose: Describes the details of a project.",
        ),
    ),
    (
        "DescribeReservedCapacity",
        Decision::Deferred(
            "Required selectors ReservedCapacityArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ReservedCapacityArn, ReservedCapacityType, Status, AvailabilityZone, DurationHours, DurationMinutes, StartTime, EndTime, InstanceType, TotalInstanceCount, AvailableInstanceCount, InUseInstanceCount, UltraServerSummary. Pinned purpose: Retrieves details about a reserved capacity.",
        ),
    ),
    (
        "DescribeSpace",
        Decision::Deferred(
            "Required selectors DomainId, SpaceName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainId, SpaceArn, SpaceName, HomeEfsFileSystemUid, Status, LastModifiedTime, CreationTime, FailureReason, SpaceSettings, OwnershipSettings, SpaceSharingSettings, SpaceDisplayName, Url. Pinned purpose: Describes the space.",
        ),
    ),
    (
        "DescribeStudioLifecycleConfig",
        Decision::Deferred(
            "Required selectors StudioLifecycleConfigName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StudioLifecycleConfigArn, StudioLifecycleConfigName, CreationTime, LastModifiedTime, StudioLifecycleConfigContent, StudioLifecycleConfigAppType. Pinned purpose: Describes the Amazon SageMaker AI Studio Lifecycle Configuration.",
        ),
    ),
    (
        "DescribeSubscribedWorkteam",
        Decision::Deferred(
            "Required selectors WorkteamArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SubscribedWorkteam. Pinned purpose: Gets information about a work team provided by a vendor. It returns details about the subscription with a vendor in the Amazon Web Services Marketplace.",
        ),
    ),
    (
        "DescribeTrainingJob",
        Decision::Deferred(
            "Required selectors TrainingJobName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrainingJobName, TrainingJobArn, TuningJobArn, LabelingJobArn, AutoMLJobArn, ModelArtifacts, TrainingJobStatus, SecondaryStatus, FailureReason, HyperParameters, AlgorithmSpecification, RoleArn, InputDataConfig, OutputDataConfig, ResourceConfig, WarmPoolStatus, VpcConfig, StoppingCondition, CreationTime, TrainingStartTime, TrainingEndTime, LastModifiedTime, SecondaryStatusTransitions, FinalMetricDataList, EnableNetworkIsolation, EnableInterContainerTrafficEncryption, EnableManagedSpotTraining, CheckpointConfig, TrainingTimeInSeconds, BillableTimeInSeconds, BillableTokenCount, DebugHookConfig, ExperimentConfig, DebugRuleConfigurations, TensorBoardOutputConfig, DebugRuleEvaluationStatuses, ProfilerConfig, ProfilerRuleConfigurations, ProfilerRuleEvaluationStatuses, ProfilingStatus, Environment, RetryStrategy, RemoteDebugConfig, InfraCheckConfig, ServerlessJobConfig, MlflowConfig, ModelPackageConfig, MlflowDetails, ProgressInfo, OutputModelPackageArn. Pinned purpose: Returns information about a training job. Some of the attributes below only appear if the training job successfully starts. If the training job fails, TrainingJobStatus is Failed and, depending on the FailureReason , attributes like TrainingStartTime",
        ),
    ),
    (
        "DescribeTrainingPlan",
        Decision::Deferred(
            "Required selectors TrainingPlanName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TrainingPlanArn, TrainingPlanName, Status, StatusMessage, DurationHours, DurationMinutes, StartTime, EndTime, UpfrontFee, CurrencyCode, TotalInstanceCount, AvailableInstanceCount, InUseInstanceCount, UnhealthyInstanceCount, AvailableSpareInstanceCount, TotalUltraServerCount, TargetResources, ReservedCapacitySummaries. Pinned purpose: Retrieves detailed information about a specific training plan.",
        ),
    ),
];
