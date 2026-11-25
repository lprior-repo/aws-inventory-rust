use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateDataset",
            "CreateInferenceScheduler",
            "CreateLabel",
            "CreateLabelGroup",
            "CreateModel",
            "CreateRetrainingScheduler",
            "DeleteDataset",
            "DeleteInferenceScheduler",
            "DeleteLabel",
            "DeleteLabelGroup",
            "DeleteModel",
            "DeleteResourcePolicy",
            "DeleteRetrainingScheduler",
            "ImportDataset",
            "ImportModelVersion",
            "PutResourcePolicy",
            "StartDataIngestionJob",
            "StartInferenceScheduler",
            "StartRetrainingScheduler",
            "StopInferenceScheduler",
            "StopRetrainingScheduler",
            "TagResource",
            "UntagResource",
            "UpdateActiveModelVersion",
            "UpdateInferenceScheduler",
            "UpdateLabelGroup",
            "UpdateModel",
            "UpdateRetrainingScheduler",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeDataIngestionJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (JobId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDataset", "ListSensorStatistics"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DatasetName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeInferenceScheduler", "ListInferenceExecutions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InferenceSchedulerName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeLabel", "ListLabels"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["DescribeLabelGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (LabelGroupName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "DescribeModel",
            "DescribeRetrainingScheduler",
            "ListModelVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ModelName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeModelVersion"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ModelName, ModelVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeResourcePolicy", "ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListDataIngestionJobs"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListInferenceEvents"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (InferenceSchedulerName, IntervalStartTime, IntervalEndTime); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListDatasets",
        items: "DatasetSummaries",
        id: Id::Arn("DatasetArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["DatasetArn", "DatasetName", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListInferenceSchedulers",
        items: "InferenceSchedulerSummaries",
        id: Id::Arn("InferenceSchedulerArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "InferenceSchedulerArn",
            "ModelName",
            "ModelArn",
            "InferenceSchedulerName",
            "Status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListModels",
        items: "ModelSummaries",
        id: Id::Arn("ModelArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "ModelArn",
            "ModelName",
            "DatasetName",
            "DatasetArn",
            "Status",
            "ActiveModelVersionArn",
            "LatestScheduledRetrainingStatus",
            "RetrainingSchedulerStatus",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListLabelGroups",
        items: "LabelGroupSummaries",
        id: Id::Arn("LabelGroupArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["LabelGroupArn", "LabelGroupName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListRetrainingSchedulers",
        items: "RetrainingSchedulerSummaries",
        id: Id::Arn("ModelArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["ModelArn", "ModelName", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lookoutequipment", GROUPS, RESOURCES)
}
