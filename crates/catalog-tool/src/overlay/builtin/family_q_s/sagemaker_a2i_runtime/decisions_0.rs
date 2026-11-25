use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("DeleteHumanLoop", Decision::Denied(DenyReason::Mutating)),
    ("DescribeHumanLoop", Decision::Denied(DenyReason::Content)),
    ("ListHumanLoops", Decision::Denied(DenyReason::Content)),
    ("StartHumanLoop", Decision::Denied(DenyReason::Paid)),
    ("StopHumanLoop", Decision::Denied(DenyReason::Mutating)),
];
