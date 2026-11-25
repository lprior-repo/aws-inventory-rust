use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("BatchGetRecord", Decision::Denied(DenyReason::Paid)),
    ("BatchWriteRecord", Decision::Denied(DenyReason::Content)),
    ("DeleteRecord", Decision::Denied(DenyReason::Mutating)),
    ("GetRecord", Decision::Denied(DenyReason::Paid)),
    ("ListRecords", Decision::Denied(DenyReason::Content)),
    ("PutRecord", Decision::Denied(DenyReason::Mutating)),
    ("UpdateRecord", Decision::Denied(DenyReason::Mutating)),
];
