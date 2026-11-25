use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateConnector",
            "CreateDirectoryRegistration",
            "CreateServicePrincipalName",
            "CreateTemplate",
            "CreateTemplateGroupAccessControlEntry",
            "DeleteConnector",
            "DeleteDirectoryRegistration",
            "DeleteServicePrincipalName",
            "DeleteTemplate",
            "DeleteTemplateGroupAccessControlEntry",
            "TagResource",
            "UntagResource",
            "UpdateTemplate",
            "UpdateTemplateGroupAccessControlEntry",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetConnector"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ConnectorArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDirectoryRegistration"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DirectoryRegistrationArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetServicePrincipalName"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (DirectoryRegistrationArn, ConnectorArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplate"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TemplateArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplateGroupAccessControlEntry"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TemplateArn, GroupSecurityIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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
        operation: "ListConnectors",
        items: "Connectors",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "CertificateAuthorityArn", "DirectoryId", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListDirectoryRegistrations",
        items: "DirectoryRegistrations",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "DirectoryId", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListTemplates",
        items: "Templates",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ConnectorArn", "Name", "Status"],
        tags: None,
        bindings: &[("ConnectorArn", "ListConnectors", "Arn")],
        fixed: &[],
    },
    Resource {
        operation: "ListTemplateGroupAccessControlEntries",
        items: "AccessControlEntries",
        id: Id::Correlated(&["TemplateArn"], &["GroupSecurityIdentifier"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["TemplateArn", "GroupSecurityIdentifier", "GroupDisplayName"],
        tags: None,
        bindings: &[("TemplateArn", "ListTemplates", "Arn")],
        fixed: &[],
    },
    Resource {
        operation: "ListServicePrincipalNames",
        items: "ServicePrincipalNames",
        id: Id::Correlated(&["DirectoryRegistrationArn"], &["ConnectorArn"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["DirectoryRegistrationArn", "ConnectorArn", "Status"],
        tags: None,
        bindings: &[(
            "DirectoryRegistrationArn",
            "ListDirectoryRegistrations",
            "Arn",
        )],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pca-connector-ad", GROUPS, RESOURCES)
}
