// Pinned operation purposes: .local/aws-models/models/sagemaker-metrics/service/2022-09-30/sagemaker-metrics-2022-09-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchGetMetrics",
        Decision::Deferred(
            "Required selectors MetricQueries are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MetricQueryResults. Pinned purpose: Used to retrieve training metrics from SageMaker.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Used to ingest training metrics into SageMaker. These metrics can be visualized in SageMaker Studio.
    ("BatchPutMetrics", Decision::Denied(DenyReason::Mutating)),
];
