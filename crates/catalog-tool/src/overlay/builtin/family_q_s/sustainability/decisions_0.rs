// Pinned operation purposes: .local/aws-models/models/sustainability/service/2018-05-10/sustainability-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns estimated carbon emission values based on customer grouping and filtering parameters. We recommend using pagination to ensure that the operation returns quickly and successfully.
    ("GetEstimatedCarbonEmissions", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the possible dimension values available for a customer's account. We recommend using pagination to ensure that the operation returns quickly and successfully.
    ("GetEstimatedCarbonEmissionsDimensionValues", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns estimated water allocation values based on customer grouping and filtering parameters. We recommend using pagination to ensure that the operation returns quickly and successfully.
    ("GetEstimatedWaterAllocation", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the possible dimension values available for a customer's account. We recommend using pagination to ensure that the operation returns quickly and successfully.
    ("GetEstimatedWaterAllocationDimensionValues", Decision::Denied(DenyReason::Content)),
];
