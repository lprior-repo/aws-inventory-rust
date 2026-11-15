// Pinned operation purposes: .local/aws-models/models/sagemaker-a2i-runtime/service/2019-11-07/sagemaker-a2i-runtime-2019-11-07.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified human loop for a flow definition. If the human loop was deleted, this operation will return a ResourceNotFoundException .
    ("DeleteHumanLoop", Decision::Denied(DenyReason::Mutating)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns information about the specified human loop. If the human loop was deleted, this operation will return a ResourceNotFoundException error.
    ("DescribeHumanLoop", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns information about human loops, given the specified parameters. If a human loop was deleted, it will not be included.
    ("ListHumanLoops", Decision::Denied(DenyReason::Content)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Starts a human loop, provided that at least one activation condition is met.
    ("StartHumanLoop", Decision::Denied(DenyReason::Paid)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops the specified human loop.
    ("StopHumanLoop", Decision::Denied(DenyReason::Mutating)),
];
