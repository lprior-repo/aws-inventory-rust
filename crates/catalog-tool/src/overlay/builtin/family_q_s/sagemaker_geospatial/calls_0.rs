// Pinned model: .local/aws-models/models/sagemaker-geospatial/service/2020-05-27/sagemaker-geospatial-2020-05-27.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListEarthObservationJobs",
        items: "EarthObservationJobSummaries",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListVectorEnrichmentJobs",
        items: "VectorEnrichmentJobSummaries",
        identity: ResourceIdentity::Arn("Arn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
];
