// Pinned operation purposes: .local/aws-models/models/sagemaker/service/2017-07-24/sagemaker-2017-07-24.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon SageMaker Model Card export job.
    ("CreateModelCardExportJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates the definition for a model explainability job.
    ("CreateModelExplainabilityJobDefinition", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a model package that you can use to create SageMaker models or list on Amazon Web Services Marketplace, or a versioned model that is part of a model group. Buyers can subscribe to model packages listed on Amazon Web Services Marketplace to create models in SageMaker. To create a model package by specifying a Docker container that contains your inference code and the Amazon S3 location of y
    ("CreateModelPackage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a model group. A model group contains a group of model versions.
    ("CreateModelPackageGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a definition for a job that monitors model quality and drift. For information about model monitor, see Amazon SageMaker AI Model Monitor .
    ("CreateModelQualityJobDefinition", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a schedule that regularly starts Amazon SageMaker AI Processing Jobs to monitor the data captured for an Amazon SageMaker AI Endpoint.
    ("CreateMonitoringSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an SageMaker AI notebook instance. A notebook instance is a machine learning (ML) compute instance running on a Jupyter notebook. In a CreateNotebookInstance request, specify the type of ML compute instance that you want to run. SageMaker AI launches the instance, installs common libraries that you can use to explore datasets for model training, and attaches an ML storage volume to the not
    ("CreateNotebookInstance", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a lifecycle configuration that you can associate with a notebook instance. A lifecycle configuration is a collection of shell scripts that run when you create or start a notebook instance. Each lifecycle configuration script has a limit of 16384 characters. The value of the $PATH environment variable that is available to both scripts is /sbin:bin:/usr/sbin:/usr/bin . View Amazon CloudWatch
    ("CreateNotebookInstanceLifecycleConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a job that optimizes a model for inference performance. To create the job, you provide the location of a source model, and you provide the settings for the optimization techniques that you want the job to apply. When the job completes successfully, SageMaker uploads the new optimized model to the output destination that you specify. For more information about how to use this action, and ab
    ("CreateOptimizationJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon SageMaker Partner AI App.
    ("CreatePartnerApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a presigned URL to access an Amazon SageMaker Partner AI App.
    ("CreatePartnerAppPresignedUrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a pipeline using a JSON pipeline definition.
    ("CreatePipeline", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a URL for a specified UserProfile in a Domain. When accessed in a web browser, the user will be automatically signed in to the domain, and granted access to all of the Apps and files associated with the Domain's Amazon Elastic File System volume. This operation can only be called when the authentication mode equals IAM. The IAM role or user passed to this API defines the permissions to acc
    ("CreatePresignedDomainUrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Returns a presigned URL that you can use to connect to the MLflow UI attached to your MLflow App. For more information, see Launch the MLflow UI using a presigned URL .
    ("CreatePresignedMlflowAppUrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Returns a presigned URL that you can use to connect to the MLflow UI attached to your tracking server. For more information, see Launch the MLflow UI using a presigned URL .
    ("CreatePresignedMlflowTrackingServerUrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Returns a URL that you can use to connect to the Jupyter server from a notebook instance. In the SageMaker AI console, when you choose Open next to a notebook instance, SageMaker AI opens a new tab showing the Jupyter server home page from the notebook instance. The console uses this API to get the URL and show the page. The IAM role or user used to call this API defines the permissions to access 
    ("CreatePresignedNotebookInstanceUrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a processing job.
    ("CreateProcessingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a machine learning (ML) project that can contain one or more templates that set up an ML pipeline from training to deploying an approved model.
    ("CreateProject", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a private space or a space used for real time collaboration in a domain.
    ("CreateSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon SageMaker AI Studio Lifecycle Configuration.
    ("CreateStudioLifecycleConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a model training job. After training completes, SageMaker saves the resulting model artifacts to an Amazon S3 location that you specify. If you choose to host your model using SageMaker hosting services, you can use the resulting model artifacts as part of the model. You can also use the artifacts in a machine learning service other than SageMaker, provided that you know how to use them for
    ("CreateTrainingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new training plan in SageMaker to reserve compute capacity. Amazon SageMaker Training Plan is a capability within SageMaker that allows customers to reserve and manage GPU capacity for large-scale AI model training. It provides a way to secure predictable access to computational resources within specific timelines and budgets, without the need to manage underlying infrastructure. How it 
    ("CreateTrainingPlan", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a training session in Amazon SageMaker. A training session provisions compute resources for fine-tuning a foundation model using Low-Rank Adaptation (LoRA). After the session is created, you can submit training operations and sample inference requests through the Amazon SageMaker Training Session Runtime API. The following operations are related to CreateTrainingSession : DescribeTrainingS
    ("CreateTrainingSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a transform job. A transform job uses a trained model to get inferences on a dataset and saves these results to an Amazon S3 location that you specify. To perform batch transformations, you create a transform job and use the data that you have readily available. In the request body, you provide the following: TransformJobName - Identifies the transform job. The name must be unique within an
    ("CreateTransformJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an SageMaker trial . A trial is a set of steps called trial components that produce a machine learning model. A trial is part of a single SageMaker experiment . When you use SageMaker Studio or the SageMaker Python SDK, all experiments, trials, and trial components are automatically tracked, logged, and indexed. When you use the Amazon Web Services SDK for Python (Boto), you must use the l
    ("CreateTrial", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a trial component , which is a stage of a machine learning trial . A trial is composed of one or more trial components. A trial component can be used in multiple trials. Trial components include pre-processing jobs, training jobs, and batch transform jobs. When you use SageMaker Studio or the SageMaker Python SDK, all experiments, trials, and trial components are automatically tracked, log
    ("CreateTrialComponent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a user profile. A user profile represents a single user within a domain, and is the main way to reference a "person" for the purposes of sharing, reporting, and other user-oriented features. This entity is created when a user onboards to a domain. If an administrator invites a person by email or imports them from IAM Identity Center, a user profile is automatically created. A user profile 
    ("CreateUserProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to create a workforce. This operation will return an error if a workforce already exists in the Amazon Web Services Region that you specify. You can only create one workforce in each Amazon Web Services Region per Amazon Web Services account. If you want to create a new workforce in an Amazon Web Services Region where a workforce already exists, use the DeleteWorkforce API opera
    ("CreateWorkforce", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new work team for labeling your data. A work team is defined by one or more Amazon Cognito user pools. You must first create the user pools before you can create a work team. You cannot create more than 25 work teams in an account and region.
    ("CreateWorkteam", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified AI benchmark job.
    ("DeleteAIBenchmarkJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified AI recommendation job.
    ("DeleteAIRecommendationJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified AI workload configuration. You cannot delete a configuration that is referenced by an active benchmark job.
    ("DeleteAIWorkloadConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an action.
    ("DeleteAction", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified algorithm from your account.
    ("DeleteAlgorithm", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Used to stop and delete an app.
    ("DeleteApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an AppImageConfig.
    ("DeleteAppImageConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an artifact. Either ArtifactArn or Source must be specified.
    ("DeleteArtifact", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an association.
    ("DeleteAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a SageMaker HyperPod cluster.
    ("DeleteCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an extension from a SageMaker HyperPod cluster. SageMaker HyperPod removes the extension's software from every node in the cluster. While SageMaker HyperPod removes the extension, its status is Deleting . After SageMaker HyperPod removes the extension from all nodes, it no longer appears in ListClusterExtensions , and DescribeClusterExtension returns a ResourceNotFound error. Deleting an e
    ("DeleteClusterExtension", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the cluster policy of the cluster.
    ("DeleteClusterSchedulerConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified Git repository from your account.
    ("DeleteCodeRepository", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified compilation job. This action deletes only the compilation job resource in Amazon SageMaker AI. It doesn't delete other resources that are related to that job, such as the model artifacts that the job creates, the compilation logs in CloudWatch, the compiled model, or the IAM role. You can delete a compilation job only if its current status is COMPLETED , FAILED , or STOPPED .
    ("DeleteCompilationJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the compute allocation from the cluster.
    ("DeleteComputeQuota", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an context.
    ("DeleteContext", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a data quality monitoring job definition.
    ("DeleteDataQualityJobDefinition", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a fleet.
    ("DeleteDeviceFleet", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Used to delete a domain. If you onboarded with IAM mode, you will need to delete your domain to onboard again using IAM Identity Center. Use with caution. All of the members of the domain will lose access to their EFS volume, including data, notebooks, and other artifacts.
    ("DeleteDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an edge deployment plan if (and only if) all the stages in the plan are inactive or there are no stages in the plan.
    ("DeleteEdgeDeploymentPlan", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a stage in an edge deployment plan if (and only if) the stage is inactive.
    ("DeleteEdgeDeploymentStage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an endpoint. SageMaker frees up all of the resources that were deployed when the endpoint was created. SageMaker retires any custom KMS key grants associated with the endpoint, meaning you don't need to use the RevokeGrant API call. When you delete your endpoint, SageMaker asynchronously deletes associated endpoint resources such as KMS key grants. You might still see these resources in yo
    ("DeleteEndpoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an endpoint configuration. The DeleteEndpointConfig API deletes only the specified configuration. It does not delete endpoints created using the configuration. You must not delete an EndpointConfig in use by an endpoint that is live or while the UpdateEndpoint or CreateEndpoint operations are being performed on the endpoint. If you delete the EndpointConfig of an endpoint that is active or
    ("DeleteEndpointConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an SageMaker experiment. All trials associated with the experiment must be deleted first. Use the ListTrials API to get a list of the trials associated with the experiment.
    ("DeleteExperiment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete the FeatureGroup and any data that was written to the OnlineStore of the FeatureGroup . Data cannot be accessed from the OnlineStore immediately after DeleteFeatureGroup is called. Data written into the OfflineStore will not be deleted. The Amazon Web Services Glue database and tables that are automatically created for your OfflineStore are not deleted. Note that it can take approximately 1
    ("DeleteFeatureGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified flow definition.
    ("DeleteFlowDefinition", Decision::Denied(DenyReason::Mutating)),
];
