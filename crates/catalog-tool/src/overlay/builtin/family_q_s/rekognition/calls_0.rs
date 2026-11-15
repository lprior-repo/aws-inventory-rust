// Pinned model: .local/aws-models/models/rekognition/service/2016-06-27/rekognition-2016-06-27.json
use super::super::construction::{Inventory, ResourceIdentity, Input};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListCollections",
        items: "CollectionIds",
        identity: ResourceIdentity::Key(&["$"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "DescribeProjects",
        items: "ProjectDescriptions",
        identity: ResourceIdentity::Arn("ProjectArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[Input::Strings("Features", &["CUSTOM_LABELS", "CONTENT_MODERATION"])],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListStreamProcessors",
        items: "StreamProcessors",
        identity: ResourceIdentity::Key(&["Name"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListMediaAnalysisJobs",
        items: "MediaAnalysisJobs",
        identity: ResourceIdentity::Key(&["JobId"]),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
