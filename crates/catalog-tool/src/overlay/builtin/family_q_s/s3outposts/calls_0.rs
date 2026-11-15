// Pinned model: .local/aws-models/models/s3outposts/service/2017-07-25/s3outposts-2017-07-25.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListEndpoints",
        items: "Endpoints",
        identity: ResourceIdentity::Arn("EndpointArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListOutpostsWithS3",
        items: "Outposts",
        identity: ResourceIdentity::Arn("S3OutpostArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
