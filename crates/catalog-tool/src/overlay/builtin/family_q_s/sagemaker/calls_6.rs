// Pinned model: .local/aws-models/models/sagemaker/service/2017-07-24/sagemaker-2017-07-24.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListCodeRepositories",
        items: "CodeRepositorySummaryList",
        identity: ResourceIdentity::Arn("CodeRepositoryArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListInferenceRecommendationsJobs",
        items: "InferenceRecommendationsJobs",
        identity: ResourceIdentity::Arn("JobArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
