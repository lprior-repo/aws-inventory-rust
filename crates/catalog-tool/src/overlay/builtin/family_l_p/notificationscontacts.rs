use super::super::exclude_members;
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "ActivateEmailContact",
            "CreateEmailContact",
            "DeleteEmailContact",
            "SendActivationCode",
            "TagResource",
            "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (arn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListEmailContacts",
        items: "emailContacts",
        id: Id::Arn("arn"),
        scope: Scope::Account,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetEmailContact",
        items: "emailContact",
        id: Id::Arn("arn"),
        scope: Scope::Account,
        tokens: None,
        query: &["arn", "name", "status"],
        tags: None,
        bindings: &[("arn", "ListEmailContacts", "arn")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut service = compile("notificationscontacts", GROUPS, RESOURCES)?;
    exclude_members(&mut service, "GetEmailContact", &["emailContact.address"])?;
    exclude_members(
        &mut service,
        "ListEmailContacts",
        &["emailContacts.address"],
    )?;
    Ok(service)
}
