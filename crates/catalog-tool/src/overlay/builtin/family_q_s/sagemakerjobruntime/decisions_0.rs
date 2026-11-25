use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CompleteRollout",
        Decision::Deferred(
            "Required selectors JobArn, TrajectoryId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Marks a rollout as complete, indicating that no further turns will be appended to the trajectory. After calling this operation, the trajectory is sealed and eligible for reward submission via the UpdateReward operation.",
        ),
    ),
    ("Sample", Decision::Denied(DenyReason::Mutating)),
    (
        "SampleWithResponseStream",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateReward", Decision::Denied(DenyReason::Mutating)),
];
