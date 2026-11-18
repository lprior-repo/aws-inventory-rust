// Pinned Smithy source: .local/aws-models/models/pca-connector-scep/service/2018-05-10/pca-connector-scep-2018-05-10.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateChallenge",
            "CreateConnector",
            "DeleteChallenge",
            "DeleteConnector",
            "TagResource",
            "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetChallengeMetadata"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ChallengeArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetChallengePassword"],
        class: OverlayClass::Deny(DenyReason::Secret),
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
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists the connectors belonging to your Amazon Web Services account.
    Resource {
        operation: "ListConnectors",
        items: "Connectors",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "CertificateAuthorityArn", "Type", "Status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves the challenge metadata for the specified ARN.
    Resource {
        operation: "ListChallengeMetadata",
        items: "Challenges",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ConnectorArn"],
        tags: None,
        bindings: &[("ConnectorArn", "ListConnectors", "Arn")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pca-connector-scep", GROUPS, RESOURCES)
}
