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

const DEFERRED: &[&str] = &["GetPendingJobExecutions"];

const MUTATING: &[&str] = &[
    "StartCommandExecution",
    "StartNextPendingJobExecution",
    "UpdateJobExecution",
];

const CONTENT: &[&str] = &["DescribeJobExecution"];
