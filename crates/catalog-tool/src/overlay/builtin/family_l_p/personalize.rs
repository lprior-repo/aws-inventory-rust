// Pinned Smithy source: .local/aws-models/models/personalize/service/2018-05-22/personalize-2018-05-22.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateBatchInferenceJob",
            "CreateBatchSegmentJob",
            "CreateCampaign",
            "CreateDataDeletionJob",
            "CreateDataset",
            "CreateDatasetExportJob",
            "CreateDatasetGroup",
            "CreateDatasetImportJob",
            "CreateEventTracker",
            "CreateFilter",
            "CreateMetricAttribution",
            "CreateRecommender",
            "CreateSchema",
            "CreateSolution",
            "CreateSolutionVersion",
            "DeleteCampaign",
            "DeleteDataset",
            "DeleteDatasetGroup",
            "DeleteEventTracker",
            "DeleteFilter",
            "DeleteMetricAttribution",
            "DeleteRecommender",
            "DeleteSchema",
            "DeleteSolution",
            "StartRecommender",
            "StopRecommender",
            "StopSolutionVersionCreation",
            "TagResource",
            "UntagResource",
            "UpdateCampaign",
            "UpdateDataset",
            "UpdateMetricAttribution",
            "UpdateRecommender",
            "UpdateSolution",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeAlgorithm"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (algorithmArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeBatchInferenceJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (batchInferenceJobArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeBatchSegmentJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (batchSegmentJobArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeCampaign"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (campaignArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDataDeletionJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (dataDeletionJobArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDataset"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (datasetArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDatasetExportJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (datasetExportJobArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDatasetGroup"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (datasetGroupArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeDatasetImportJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (datasetImportJobArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeEventTracker"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (eventTrackerArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeFeatureTransformation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (featureTransformationArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeFilter"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (filterArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeMetricAttribution"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (metricAttributionArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeRecipe"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (recipeArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeRecommender"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (recommenderArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeSchema"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["DescribeSolution"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (solutionArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeSolutionVersion", "GetSolutionMetrics"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (solutionVersionArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListBatchInferenceJobs",
            "ListBatchSegmentJobs",
            "ListDataDeletionJobs",
            "ListDatasetExportJobs",
            "ListDatasetImportJobs",
            "ListMetricAttributionMetrics",
            "ListRecipes",
            "ListSolutionVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Returns a list of campaigns that use the given solution. When a solution is not specified, all the campaigns associated with the account are listed. The response provides the prope
    Resource {
        operation: "ListCampaigns",
        items: "campaigns",
        id: Id::Arn("campaignArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["campaignArn", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of dataset groups. The response provides the properties for each dataset group, including the Amazon Resource Name (ARN). For more information on dataset groups, see
    Resource {
        operation: "ListDatasetGroups",
        items: "datasetGroups",
        id: Id::Arn("datasetGroupArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["datasetGroupArn", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns the list of datasets contained in the given dataset group. The response provides the properties for each dataset, including the Amazon Resource Name (ARN). For more informa
    Resource {
        operation: "ListDatasets",
        items: "datasets",
        id: Id::Arn("datasetArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["datasetArn", "name", "datasetType", "status"],
        tags: None,
        bindings: &[("datasetGroupArn", "ListDatasetGroups", "datasetGroupArn")],
        fixed: &[],
    },
    // Returns the list of event trackers associated with the account. The response provides the properties for each event tracker, including the Amazon Resource Name (ARN) and tracking I
    Resource {
        operation: "ListEventTrackers",
        items: "eventTrackers",
        id: Id::Arn("eventTrackerArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["eventTrackerArn", "name", "status"],
        tags: None,
        bindings: &[("datasetGroupArn", "ListDatasetGroups", "datasetGroupArn")],
        fixed: &[],
    },
    // Lists all filters that belong to a given dataset group.
    Resource {
        operation: "ListFilters",
        items: "Filters",
        id: Id::Arn("filterArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["filterArn", "name", "datasetGroupArn", "status"],
        tags: None,
        bindings: &[("datasetGroupArn", "ListDatasetGroups", "datasetGroupArn")],
        fixed: &[],
    },
    // Returns a list of recommenders in a given Domain dataset group. When a Domain dataset group is not specified, all the recommenders associated with the account are listed. The respo
    Resource {
        operation: "ListRecommenders",
        items: "recommenders",
        id: Id::Arn("recommenderArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "recommenderArn",
            "name",
            "datasetGroupArn",
            "recipeArn",
            "status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns the list of schemas associated with the account. The response provides the properties for each schema, including the Amazon Resource Name (ARN). For more information on sch
    Resource {
        operation: "ListSchemas",
        items: "schemas",
        id: Id::Arn("schemaArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["schemaArn", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of solutions in a given dataset group. When a dataset group is not specified, all the solutions associated with the account are listed. The response provides the pro
    Resource {
        operation: "ListSolutions",
        items: "solutions",
        id: Id::Arn("solutionArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["solutionArn", "name", "status", "recipeArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists metric attributions.
    Resource {
        operation: "ListMetricAttributions",
        items: "metricAttributions",
        id: Id::Arn("metricAttributionArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["metricAttributionArn", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("personalize", GROUPS, RESOURCES)
}
