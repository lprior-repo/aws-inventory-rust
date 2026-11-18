// Pinned operation purposes: .local/aws-models/models/sagemaker/service/2017-07-24/sagemaker-2017-07-24.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Updates the properties of a SageMaker AI image. To change the image's tags, use the AddTags and DeleteTags APIs.
    ("UpdateImage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the properties of a SageMaker AI image version.
    ("UpdateImageVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an inference component.
    (
        "UpdateInferenceComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Runtime settings for a model that is deployed with an inference component.
    (
        "UpdateInferenceComponentRuntimeConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an inference experiment that you created. The status of the inference experiment has to be either Created , Running . For more information on the status of an inference experiment, see DescribeInferenceExperiment .
    (
        "UpdateInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an MLflow App.
    ("UpdateMlflowApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates properties of an existing MLflow Tracking Server.
    (
        "UpdateMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update an Amazon SageMaker Model Card. You cannot update both model card content and model card status in a single call.
    ("UpdateModelCard", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a versioned model.
    ("UpdateModelPackage", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Update the parameters of a model monitor alert.
    (
        "UpdateMonitoringAlert",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a previously created schedule.
    (
        "UpdateMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a notebook instance. NotebookInstance updates include upgrading or downgrading the ML compute instance used for your notebook instance to accommodate changes in your workload requirements. This API can attach lifecycle configurations to notebook instances. Lifecycle configuration scripts execute with root access and the notebook instance's IAM execution role privileges. Principals with thi
    (
        "UpdateNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a notebook instance lifecycle configuration created with the CreateNotebookInstanceLifecycleConfig API. Updates to lifecycle configurations affect all notebook instances using that configuration upon their next start. Lifecycle configuration scripts execute with root access and the notebook instance's IAM execution role privileges. Grant this permission only to trusted principals. See Cust
    (
        "UpdateNotebookInstanceLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates all of the SageMaker Partner AI Apps in an account.
    ("UpdatePartnerApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a pipeline.
    ("UpdatePipeline", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a pipeline execution.
    (
        "UpdatePipelineExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a pipeline version.
    (
        "UpdatePipelineVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a machine learning (ML) project that is created from a template that sets up an ML pipeline from training to deploying an approved model. You must not update a project that is in use. If you update the ServiceCatalogProvisioningUpdateDetails of a project that is active or being created, or updated, you may lose resources already created by the project.
    ("UpdateProject", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the settings of a space. You can't edit the app type of a space in the SpaceSettings .
    ("UpdateSpace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Update a model training job to request a new Debugger profiling configuration or to change warm pool retention length.
    ("UpdateTrainingJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the display name of a trial.
    ("UpdateTrial", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates one or more properties of a trial component.
    (
        "UpdateTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a user profile.
    ("UpdateUserProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to update your workforce. You can use this operation to require that workers use specific IP addresses to work on tasks and to update your OpenID Connect (OIDC) Identity Provider (IdP) workforce configuration. The worker portal is now supported in VPC and public internet. Use SourceIpConfig to restrict worker access to tasks to a specific range of IP addresses. You specify allow
    ("UpdateWorkforce", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing work team with new member definitions or description.
    ("UpdateWorkteam", Decision::Denied(DenyReason::Mutating)),
];
