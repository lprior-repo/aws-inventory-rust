// Pinned operation purposes: .local/aws-models/models/resiliencehubv2/service/2026-02-17/resiliencehubv2-2026-02-17.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Starts a failure mode assessment.
    (
        "StartFailureModeAssessment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a run of a test. Each run scopes to the current resources in the service and produces a pass or fail outcome.
    ("StartTestRun", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops an in-progress test run.
    ("StopTestRun", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds tags to a resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a resilience assertion.
    ("UpdateAssertion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a dependency classification.
    ("UpdateDependency", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing finding.
    (
        "UpdateFailureModeFinding",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing resilience policy.
    ("UpdatePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing service.
    ("UpdateService", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a service function.
    (
        "UpdateServiceFunction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing system.
    ("UpdateSystem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration of an existing test.
    ("UpdateTest", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing user journey.
    ("UpdateUserJourney", Decision::Denied(DenyReason::Mutating)),
];
