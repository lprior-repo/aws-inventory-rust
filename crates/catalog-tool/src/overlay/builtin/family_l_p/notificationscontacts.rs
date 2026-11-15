// Pinned Smithy source: .local/aws-models/models/notificationscontacts/service/2018-05-10/notificationscontacts-2018-05-10.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "ActivateEmailContact", "CreateEmailContact", "DeleteEmailContact", 
            "SendActivationCode", "TagResource", "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
             "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (arn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists all email contacts created under the Account.
    Resource {
        operation: "ListEmailContacts",
        items: "emailContacts", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Same pinned response resource shape as ListEmailContacts; no content-bearing expansion.
    Resource {
        operation: "GetEmailContact",
        items: "emailContact", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: None,
        query: &["arn", "name", "status"],
        tags: None,
        bindings: &[("arn", "ListEmailContacts", "arn")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("notificationscontacts", GROUPS, RESOURCES)
}
