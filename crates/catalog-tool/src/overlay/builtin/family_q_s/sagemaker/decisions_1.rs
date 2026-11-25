use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CreateModelCardExportJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateModelExplainabilityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateModelPackage", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateModelPackageGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateModelQualityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateNotebookInstanceLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateOptimizationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreatePartnerApp", Decision::Denied(DenyReason::Mutating)),
    (
        "CreatePartnerAppPresignedUrl",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreatePipeline", Decision::Denied(DenyReason::Mutating)),
    (
        "CreatePresignedDomainUrl",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreatePresignedMlflowAppUrl",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreatePresignedMlflowTrackingServerUrl",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreatePresignedNotebookInstanceUrl",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateProcessingJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateProject", Decision::Denied(DenyReason::Mutating)),
    ("CreateSpace", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateStudioLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTrainingJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateTrainingPlan", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTrainingSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTransformJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateTrial", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateUserProfile", Decision::Denied(DenyReason::Mutating)),
    ("CreateWorkforce", Decision::Denied(DenyReason::Mutating)),
    ("CreateWorkteam", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAIBenchmarkJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAIRecommendationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAIWorkloadConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAction", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAlgorithm", Decision::Denied(DenyReason::Mutating)),
    ("DeleteApp", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAppImageConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteArtifact", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAssociation", Decision::Denied(DenyReason::Mutating)),
    ("DeleteCluster", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteClusterExtension",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteClusterSchedulerConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteCodeRepository",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteCompilationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteComputeQuota", Decision::Denied(DenyReason::Mutating)),
    ("DeleteContext", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteDataQualityJobDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteDeviceFleet", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteEdgeDeploymentPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEdgeDeploymentStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteEndpoint", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteEndpointConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteExperiment", Decision::Denied(DenyReason::Mutating)),
    ("DeleteFeatureGroup", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteFlowDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
];
