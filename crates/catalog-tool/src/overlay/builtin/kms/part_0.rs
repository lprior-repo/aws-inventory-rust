use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, IdentityField, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListKeys", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Keys")?),
        page_size: Some(1),
        identity: Some(Identity::Arn(path("KeyArn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("Truncated")?) }),
        ..empty()
    }),
("ListAliases", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Aliases")?),
        page_size: Some(1),
        identity: Some(Identity::Arn(path("AliasArn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("Truncated")?) }),
        ..empty()
    }),
("DescribeCustomKeyStores", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CustomKeyStores")?),
        page_size: Some(1),
        identity: Some(Identity::Key(vec![path("CustomKeyStoreId")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("Truncated")?) }),
        ..empty()
    }),
("ListGrants", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Grants")?),
        page_size: Some(1),
        identity: Some(Identity::Key(vec![path("KeyId")?, path("GrantId")?])),
        bind: [(path("KeyId")?, Binding::From { operation: name("ListKeys")?, path: path("KeyArn")? })].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("Truncated")?) }),
        ..empty()
    }),
("GetKeyLastUsage", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListKeys.KeyArn can bind KeyId, and the nonpaginated response has usage metadata, but response KeyId is not documented as a canonical ARN. Shared parent-derived identity and enrichment attachment must preserve the discovered key identity rather than create a second key. Preserve TrackingStartDate and KeyCreationDate; empty usage is not proof of lifetime nonuse.".to_owned()),
        ..empty()
    }),
("GetKeyPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("KeyId can bind ListKeys.KeyArn and PolicyName can be fixed to default, but the nonpaginated response contains only Policy and PolicyName, not a key identifier. Shared parent/request-derived identity and enrichment attachment are required to associate policy configuration with its key. Do not invent an output KeyId path.".to_owned()),
        ..empty()
    }),
("GetKeyRotationStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("KeyId can bind ListKeys.KeyArn; nonpaginated output contains KeyId and rotation metadata. Automatic rotation is restricted by key type and origin, which ListKeys does not return. Shared metadata-based eligibility selectors and parent-derived canonical identity are required; denied DescribeKey cannot supply the missing eligibility metadata.".to_owned()),
        ..empty()
    }),
("GetPublicKey", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("This downloads public, not secret, asymmetric key material plus algorithm metadata. KeyId can bind ListKeys.KeyArn, but ListKeys does not identify asymmetric keys. Shared eligibility selectors and a permitted key-type metadata source are required before calling it; denied DescribeKey must not be used to discover eligibility.".to_owned()),
        ..empty()
    }),
("ListKeyPolicies", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PolicyNames")?),
        bind: [(path("KeyId")?, Binding::From { operation: name("ListKeys")?, path: path("KeyArn")? })].into_iter().collect(),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::Request(path("KeyId")?), IdentityField::Selected(path("$")?)])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("Truncated")?) }),
        ..empty()
    }),
("ListKeyRotations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("KeyId can bind ListKeys.KeyArn; Rotations, Marker/NextMarker, Truncated and Limit are modeled. Entries expose KeyId and KeyMaterialId, with rotation metadata, not plaintext material. The operation requires rotation-capable keys; shared eligibility selectors and a permitted key-type/origin metadata source are required. IncludeKeyMaterial defaults to ROTATIONS_ONLY and must not silently become ALL_KEY_MATERIAL.".to_owned()),
        ..empty()
    }),
("ListResourceTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("KeyId can bind ListKeys.KeyArn; Tags, Marker/NextMarker, Truncated and Limit are modeled. Each item contains only TagKey and TagValue, so TagKey alone collides across keys. Shared parent-key identity projection and paginated tag enrichment attachment are required; no returned KeyId exists to use as an item identity.".to_owned()),
        ..empty()
    }),
("ListRetirableGrants", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Grants, Marker/NextMarker, Truncated and Limit are modeled, but documentation requires exactly one of RetiringPrincipal or RetiringServicePrincipal despite neither being Smithy-required. Shared account-local principal ARN enumeration/binding and exactly-one input validation are required; account ID alone is not a principal ARN, and the service-principal filter is restricted to service callers. Results can include foreign-account keys and must not be assumed local.".to_owned()),
        ..empty()
    }),
("DescribeKey", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("GetParametersForImport", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("CancelKeyDeletion", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ConnectCustomKeyStore", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateAlias", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateCustomKeyStore", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateGrant", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateKey", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
