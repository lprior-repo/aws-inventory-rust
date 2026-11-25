use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "StartFailureModeAssessment",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartTestRun", Decision::Denied(DenyReason::Mutating)),
    ("StopTestRun", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAssertion", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDependency", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFailureModeFinding",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdatePolicy", Decision::Denied(DenyReason::Mutating)),
    ("UpdateService", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateServiceFunction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSystem", Decision::Denied(DenyReason::Mutating)),
    ("UpdateTest", Decision::Denied(DenyReason::Mutating)),
    ("UpdateUserJourney", Decision::Denied(DenyReason::Mutating)),
];
