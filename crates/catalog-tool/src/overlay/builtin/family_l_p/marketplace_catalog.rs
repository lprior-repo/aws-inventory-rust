// Pinned Smithy source: .local/aws-models/models/marketplace-catalog/service/2018-09-17/marketplace-catalog-2018-09-17.json
use super::policy::{Group, compile};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "BatchDescribeEntities", "CancelChangeSet", "DeleteResourcePolicy", 
            "PutResourcePolicy", "StartChangeSet", "TagResource", "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeAssessment", "DescribeChangeSet", "DescribeEntity", "GetResourcePolicy", 
            "ListAssessments", "ListChangeSets", "ListEntities", "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Entity enumeration requires explicit EntityType dispatch across product, offer and authorization variants; a single fixed type would silently omit entity families."),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-catalog", GROUPS, &[])
}
