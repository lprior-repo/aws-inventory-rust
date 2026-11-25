use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "GetEstimatedCarbonEmissions",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetEstimatedCarbonEmissionsDimensionValues",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetEstimatedWaterAllocation",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetEstimatedWaterAllocationDimensionValues",
        Decision::Denied(DenyReason::Content),
    ),
];
