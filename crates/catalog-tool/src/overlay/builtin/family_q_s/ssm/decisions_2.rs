use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "RemoveTagsFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ResetServiceSetting",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ResumeSession", Decision::Denied(DenyReason::Mutating)),
    (
        "SendAutomationSignal",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("SendCommand", Decision::Denied(DenyReason::Mutating)),
    ("StartAccessRequest", Decision::Denied(DenyReason::Mutating)),
    (
        "StartAssociationsOnce",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartAutomationExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartChangeRequestExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartExecutionPreview",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartSession", Decision::Denied(DenyReason::Mutating)),
    (
        "StopAutomationExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TerminateSession", Decision::Denied(DenyReason::Mutating)),
    (
        "UnlabelParameterVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateAssociation", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAssociationStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateCloudConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateDocument", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDocumentDefaultVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDocumentMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMaintenanceWindowTarget",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateMaintenanceWindowTask",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateManagedInstanceRole",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateOpsItem", Decision::Denied(DenyReason::Mutating)),
    ("UpdateOpsMetadata", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdatePatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResourceDataSync",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateServiceSetting",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ValidateCloudConnector",
        Decision::Deferred(
            "Required selectors CloudConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ValidationFindings, NextToken. Pinned purpose: Validates the configuration and connectivity of a cloud connector.",
        ),
    ),
];
