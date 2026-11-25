use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("UpdateImage", Decision::Denied(DenyReason::Mutating)),
    ("UpdateImageVersion", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateInferenceComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateInferenceComponentRuntimeConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateInferenceExperiment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateMlflowApp", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateMlflowTrackingServer",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateModelCard", Decision::Denied(DenyReason::Mutating)),
    ("UpdateModelPackage", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateMonitoringAlert",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMonitoringSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateNotebookInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateNotebookInstanceLifecycleConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdatePartnerApp", Decision::Denied(DenyReason::Mutating)),
    ("UpdatePipeline", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdatePipelineExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdatePipelineVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateProject", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSpace", Decision::Denied(DenyReason::Mutating)),
    ("UpdateTrainingJob", Decision::Denied(DenyReason::Mutating)),
    ("UpdateTrial", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateTrialComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateUserProfile", Decision::Denied(DenyReason::Mutating)),
    ("UpdateWorkforce", Decision::Denied(DenyReason::Mutating)),
    ("UpdateWorkteam", Decision::Denied(DenyReason::Mutating)),
];
