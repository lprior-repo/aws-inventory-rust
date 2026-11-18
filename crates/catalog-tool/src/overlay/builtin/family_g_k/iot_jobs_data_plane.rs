//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iot-jobs-data-plane/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "iot-jobs-data-plane",
        approved,
        DEFERRED,
        "Job execution can return jobDocument; start/update operations mutate devices/jobs. Pending-job discovery still requires thing identity and data-plane endpoint binding; none is approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &["GetPendingJobExecutions"];

const MUTATING: &[&str] = &[
    "StartCommandExecution",
    "StartNextPendingJobExecution",
    "UpdateJobExecution",
];

const CONTENT: &[&str] = &["DescribeJobExecution"];
