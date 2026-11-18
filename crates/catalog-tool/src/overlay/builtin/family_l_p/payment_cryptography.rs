// Pinned Smithy source: .local/aws-models/models/payment-cryptography/service/2021-09-14/payment-cryptography-2021-09-14.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AddKeyReplicationRegions",
            "AssociateMpaTeam",
            "CreateAlias",
            "CreateKey",
            "DeleteAlias",
            "DeleteKey",
            "DeleteResourcePolicy",
            "DisableDefaultKeyReplicationRegions",
            "DisassociateMpaTeam",
            "EnableDefaultKeyReplicationRegions",
            "ExportKey",
            "ImportKey",
            "PutResourcePolicy",
            "RemoveKeyReplicationRegions",
            "RestoreKey",
            "StartKeyUsage",
            "StopKeyUsage",
            "TagResource",
            "UntagResource",
            "UpdateAlias",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetCertificateSigningRequest"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (KeyIdentifier, SigningAlgorithm, CertificateSubject); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDefaultKeyReplicationRegions"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["GetKey", "GetPublicKeyCertificate"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (KeyIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetMpaTeamAssociation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Action); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetParametersForExport"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (KeyMaterialType, SigningKeyAlgorithm); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetParametersForImport"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (KeyMaterialType, WrappingKeyAlgorithm); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetResourcePolicy", "ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists the keys in the caller's Amazon Web Services account and Amazon Web Services Region. You can filter the list of keys. This is a paginated operation, which means that each res
    Resource {
        operation: "ListKeys",
        items: "Keys",
        id: Id::Arn("KeyArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["KeyArn", "KeyState", "MultiRegionKeyType", "PrimaryRegion"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists the aliases for all keys in the caller's Amazon Web Services account and Amazon Web Services Region. You can filter the aliases by keyARN . For more information, see Using al
    Resource {
        operation: "ListAliases",
        items: "Aliases",
        id: Id::Key(&["AliasName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["AliasName", "KeyArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Same pinned response resource shape as ListAliases; no content-bearing expansion.
    Resource {
        operation: "GetAlias",
        items: "Alias",
        id: Id::Key(&["AliasName"]),
        scope: Scope::Region,
        tokens: None,
        query: &["AliasName", "KeyArn"],
        tags: None,
        bindings: &[("AliasName", "ListAliases", "AliasName")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("payment-cryptography", GROUPS, RESOURCES)
}
