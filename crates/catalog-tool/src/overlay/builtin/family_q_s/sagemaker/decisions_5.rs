// Pinned operation purposes: .local/aws-models/models/sagemaker/service/2017-07-24/sagemaker-2017-07-24.json
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
    // Changes service state rather than reading inventory. Pinned purpose: Notifies the pipeline that the execution of a callback step failed, along with a message describing why. When a callback step is run, the pipeline generates a callback token and includes the token in a message sent to Amazon Simple Queue Service (Amazon SQS).
    (
        "SendPipelineExecutionStepFailure",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Notifies the pipeline that the execution of a callback step succeeded and provides a list of the step's output parameters. When a callback step is run, the pipeline generates a callback token and includes the token in a message sent to Amazon Simple Queue Service (Amazon SQS).
    (
        "SendPipelineExecutionStepSuccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Start deep health checks for a SageMaker HyperPod cluster. You can use DescribeClusterNode API to track progress of the deep health checks. The unhealthy nodes will be automatically rebooted or replaced. Please see Resilience-related Kubernetes labels by SageMaker HyperPod for details.
    (
        "StartClusterHealthCheck",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a stage in an edge deployment plan.
    (
        "StartEdgeDeploymentStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts an inference experiment.
    (
        "StartInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Programmatically start an MLflow Tracking Server.
    (
        "StartMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a previously stopped monitoring schedule. By default, when you successfully create a new schedule, the status of a monitoring schedule is scheduled .
    (
        "StartMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Launches an ML compute instance with the latest version of the libraries and attaches your ML storage volume. After configuring the notebook instance, SageMaker AI sets the notebook instance status to InService . A notebook instance's status must be InService before you can connect to your Jupyter notebook.
    (
        "StartNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a pipeline execution.
    (
        "StartPipelineExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates a remote connection session between a local integrated development environments (IDEs) and a remote SageMaker space.
    ("StartSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running AI benchmark job.
    ("StopAIBenchmarkJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running AI recommendation job.
    (
        "StopAIRecommendationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: A method for forcing a running job to shut down.
    ("StopAutoMLJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a model compilation job. To stop a job, Amazon SageMaker AI sends the algorithm the SIGTERM signal. This gracefully shuts the job down. If the job hasn't stopped, it sends the SIGKILL signal. When it receives a StopCompilationJob request, Amazon SageMaker AI changes the CompilationJobStatus of the job to Stopping . After Amazon SageMaker stops the job, it sets the CompilationJobStatus to Sto
    ("StopCompilationJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a stage in an edge deployment plan.
    (
        "StopEdgeDeploymentStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Request to stop an edge packaging job.
    (
        "StopEdgePackagingJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running hyperparameter tuning job and all running training jobs that the tuning job launched. All model artifacts output from the training jobs are stored in Amazon Simple Storage Service (Amazon S3). All data that the training jobs write to Amazon CloudWatch Logs are still available in CloudWatch. After the tuning job moves to the Stopped state, it releases all reserved resources for the
    (
        "StopHyperParameterTuningJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops an inference experiment.
    (
        "StopInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops an Inference Recommender job.
    (
        "StopInferenceRecommendationsJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running job. When you call StopJob , Amazon SageMaker sets the job status to Stopping . After the job stops, the status changes to Stopped . Partial results may be available in the output location if the job was in progress. To delete a stopped job, call DeleteJob . The following operations are related to StopJob : CreateJob DescribeJob DeleteJob
    ("StopJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running labeling job. A job that is stopped cannot be restarted. Any results obtained before the job is stopped are placed in the Amazon S3 output bucket.
    ("StopLabelingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Programmatically stop an MLflow Tracking Server.
    (
        "StopMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a previously started monitoring schedule.
    (
        "StopMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Terminates the ML compute instance. Before terminating the instance, SageMaker AI disconnects the ML storage volume from it. SageMaker AI preserves the ML storage volume. SageMaker AI stops charging you for the ML compute instance when you call StopNotebookInstance . To access data on the ML storage volume for a notebook instance that has been terminated, call the StartNotebookInstance API. StartN
    (
        "StopNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Ends a running inference optimization job.
    (
        "StopOptimizationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a pipeline execution. Callback Step A pipeline execution won't stop while a callback step is running. When you call StopPipelineExecution on a pipeline execution with a running callback step, SageMaker Pipelines sends an additional Amazon SQS message to the specified SQS queue. The body of the SQS message contains a "Status" field which is set to "Stopping". You should add logic to your Amaz
    (
        "StopPipelineExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a processing job.
    ("StopProcessingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a training job. To stop a job, SageMaker sends the algorithm the SIGTERM signal, which delays job termination for 120 seconds. Algorithms might use this 120-second window to save the model artifacts, so the results of the training is not lost. When it receives a StopTrainingJob request, SageMaker changes the status of the job to Stopping . After SageMaker stops the job, it sets the status to
    ("StopTrainingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a running training session and sets the session status to Stopping . After the session stops, the status changes to Stopped . To delete a stopped session, use the DeleteTrainingSession operation. The following operations are related to StopTrainingSession : CreateTrainingSession DescribeTrainingSession DeleteTrainingSession
    (
        "StopTrainingSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a batch transform job. When Amazon SageMaker receives a StopTransformJob request, the status of the job changes to Stopping . After Amazon SageMaker stops the job, the status is set to Stopped . When you stop a batch transform job before it is completed, Amazon SageMaker doesn't store the job's output in Amazon S3.
    ("StopTransformJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an action.
    ("UpdateAction", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the properties of an AppImageConfig.
    (
        "UpdateAppImageConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an artifact.
    ("UpdateArtifact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a SageMaker HyperPod cluster.
    ("UpdateCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration or the version of an extension installed on a SageMaker HyperPod cluster. SageMaker HyperPod applies the change to every node in the cluster. You can change the configuration, the version, or both in one request. Configuration: You can change settings by sending ConfigurationValues . Settings that you include are changed. Settings that you omit keep their current values.
    (
        "UpdateClusterExtension",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update the cluster policy configuration.
    (
        "UpdateClusterSchedulerConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the platform software of a SageMaker HyperPod cluster for security patching. To learn how to use this API, see Update the SageMaker HyperPod platform software of a cluster . The UpgradeClusterSoftware API call may impact your SageMaker HyperPod cluster uptime and availability. Plan accordingly to mitigate potential disruptions to your workloads.
    (
        "UpdateClusterSoftware",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified Git repository with the specified values.
    (
        "UpdateCodeRepository",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update the compute allocation definition.
    ("UpdateComputeQuota", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a context.
    ("UpdateContext", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a fleet of devices.
    ("UpdateDeviceFleet", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates one or more devices in a fleet.
    ("UpdateDevices", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the default settings for new user profiles in the domain.
    ("UpdateDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deploys the EndpointConfig specified in the request to a new fleet of instances. SageMaker shifts endpoint traffic to the new instances with the updated endpoint configuration and then deletes the old instances using the previous EndpointConfig (there is no availability loss). For more information about how to control the update and traffic shifting process, see Update models in production . When
    ("UpdateEndpoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates variant weight of one or more variants associated with an existing endpoint, or capacity of one variant associated with an existing endpoint. When it receives the request, SageMaker sets the endpoint status to Updating . After updating the endpoint, it sets the status to InService . To check the status of an endpoint, use the DescribeEndpoint API.
    (
        "UpdateEndpointWeightsAndCapacities",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds, updates, or removes the description of an experiment. Updates the display name of an experiment.
    ("UpdateExperiment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the feature group by either adding features or updating the online store configuration. Use one of the following request parameters at a time while using the UpdateFeatureGroup API. You can add features for your feature group using the FeatureAdditions request parameter. Features cannot be removed from a feature group. You can update the online store configuration by using the OnlineStoreC
    ("UpdateFeatureGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the description and parameters of the feature group.
    (
        "UpdateFeatureMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update a hub.
    ("UpdateHub", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates SageMaker hub content (either a Model or Notebook resource). You can update the metadata that describes the resource. In addition to the required request fields, specify at least one of the following fields to update: HubContentDescription HubContentDisplayName HubContentMarkdown HubContentSearchKeywords SupportStatus For more information about hubs, see Private curated hubs for foundation
    ("UpdateHubContent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the contents of a SageMaker hub for a ModelReference resource. A ModelReference allows you to access public SageMaker JumpStart models from within your private hub. When using this API, you can update the MinVersion field for additional flexibility in the model version. You shouldn't update any additional fields when using this API, because the metadata in your private hub should match the
    (
        "UpdateHubContentReference",
        Decision::Denied(DenyReason::Mutating),
    ),
];
