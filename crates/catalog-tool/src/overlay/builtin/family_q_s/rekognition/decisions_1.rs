use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "StartContentModeration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartFaceDetection", Decision::Denied(DenyReason::Mutating)),
    ("StartFaceSearch", Decision::Denied(DenyReason::Mutating)),
    (
        "StartLabelDetection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartMediaAnalysisJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartPersonTracking",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartProjectVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartSegmentDetection",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartTextDetection", Decision::Denied(DenyReason::Mutating)),
    ("StopProjectVersion", Decision::Denied(DenyReason::Mutating)),
    (
        "StopStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDatasetEntries",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
];
