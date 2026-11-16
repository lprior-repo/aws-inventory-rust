use crate::overlay::{NameError, OperationOverlay, Binding, FixedValue, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path, name};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetTrustStore", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrustStore")?),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        bind: [(path("Identifier")?, Binding::From { operation: name("ListTrustStores")?, path: path("Id")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListVpcOrigins", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("VpcOriginList.Items")?),
        fixed: [(path("MaxItems")?, FixedValue::Integer(100))].into_iter().collect(),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("VpcOriginList.NextMarker")?, optional: false }], truncated: Some(path("VpcOriginList.IsTruncated")?) }),
        ..empty()
    }),
("GetVpcOrigin", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("VpcOrigin")?),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        bind: [(path("Id")?, Binding::From { operation: name("ListVpcOrigins")?, path: path("Id")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetDistribution", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetDistributionConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributionsByAnycastIpListId", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributionsByConnectionFunction", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributionsByConnectionMode", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributionsByRealtimeLogConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributionsByTrustStore", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListDistributionsByWebACLId", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetPublicKey", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetPublicKeyConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListPublicKeys", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetFunction", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetConnectionFunction", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetManagedCertificateDetails", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListConflictingAliases", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("ListDomainConflicts", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    })
    ])
}
