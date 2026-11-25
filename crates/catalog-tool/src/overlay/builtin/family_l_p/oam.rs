use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateLink",
            "CreateSink",
            "DeleteLink",
            "DeleteSink",
            "PutSinkPolicy",
            "TagResource",
            "UntagResource",
            "UpdateLink",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetLink", "GetSink"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Identifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetSinkPolicy", "ListAttachedLinks"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (SinkIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListLinks",
        items: "Items",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "SinkArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListSinks",
        items: "Items",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("oam", GROUPS, RESOURCES)
}
