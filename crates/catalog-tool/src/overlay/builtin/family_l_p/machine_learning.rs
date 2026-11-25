use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AddTags",
            "CreateBatchPrediction",
            "CreateDataSourceFromRDS",
            "CreateDataSourceFromRedshift",
            "CreateDataSourceFromS3",
            "CreateEvaluation",
            "CreateMLModel",
            "CreateRealtimeEndpoint",
            "DeleteBatchPrediction",
            "DeleteDataSource",
            "DeleteEvaluation",
            "DeleteMLModel",
            "DeleteRealtimeEndpoint",
            "DeleteTags",
            "UpdateBatchPrediction",
            "UpdateDataSource",
            "UpdateEvaluation",
            "UpdateMLModel",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeDataSources"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["DescribeTags"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceId, ResourceType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetBatchPrediction"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (BatchPredictionId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDataSource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DataSourceId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetEvaluation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (EvaluationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetMLModel"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (MLModelId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["Predict"],
        class: OverlayClass::Deny(DenyReason::Paid),
        reason: None,
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "DescribeMLModels",
        items: "Results",
        id: Id::Key(&["MLModelId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "MLModelId",
            "TrainingDataSourceId",
            "Name",
            "Status",
            "MLModelType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "DescribeEvaluations",
        items: "Results",
        id: Id::Key(&["EvaluationId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "EvaluationId",
            "MLModelId",
            "EvaluationDataSourceId",
            "Name",
            "Status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "DescribeBatchPredictions",
        items: "Results",
        id: Id::Key(&["BatchPredictionId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "BatchPredictionId",
            "MLModelId",
            "BatchPredictionDataSourceId",
            "Name",
            "Status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("machine-learning", GROUPS, RESOURCES)
}
