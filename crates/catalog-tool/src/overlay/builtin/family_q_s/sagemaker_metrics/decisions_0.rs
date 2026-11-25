use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchGetMetrics",
        Decision::Deferred(
            "Required selectors MetricQueries are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MetricQueryResults. Pinned purpose: Used to retrieve training metrics from SageMaker.",
        ),
    ),
    ("BatchPutMetrics", Decision::Denied(DenyReason::Mutating)),
];
