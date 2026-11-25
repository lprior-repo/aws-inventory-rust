use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateWebFunction",
            "CreateWebFunctionEndpoint",
            "CreateWebFunctionRevision",
            "DeleteResourcePolicy",
            "DeleteWebFunction",
            "DeleteWebFunctionEndpoint",
            "DeleteWebFunctionRevision",
            "PutResourcePolicy",
            "TagResource",
            "UntagResource",
            "UpdateWebFunctionEndpoint",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetResourcePolicy",
            "GetWebAccountSettings",
            "GetWebFunction",
            "GetWebFunctionEndpoint",
            "GetWebFunctionRevision",
            "ListTags",
            "ListWebFunctionEndpoints",
            "ListWebFunctionRevisions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Revision/endpoint detail requires function context; function summaries do not authorize code or revision content reads.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "ListWebFunctions",
    items: "functions",
    id: Id::Arn("functionArn"),
    scope: Scope::Region,
    tokens: Some(("nextToken", "nextToken")),
    query: &["functionArn", "functionName", "state"],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lambda-web", GROUPS, RESOURCES)
}
