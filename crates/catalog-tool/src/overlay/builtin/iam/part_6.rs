use crate::overlay::{NameError, OperationOverlay, Binding, FixedValue, Identity, IdentityField, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path, name};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListSSHPublicKeys", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("SSHPublicKeys")?),
        page_size: Some(1000),
        bind: [(path("UserName")?, Binding::From { operation: name("ListUsers")?, path: path("UserName")? })].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("UserName")?, path("SSHPublicKeyId")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListServerCertificateTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires owner identity from ListServerCertificates.ServerCertificateName and paginated tag merge into the certificate metadata record.".to_owned()),
        ..empty()
    }),
("ListServerCertificates", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ServerCertificateMetadataList")?),
        page_size: Some(1000),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListServiceSpecificCredentials", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ServiceSpecificCredentials")?),
        fixed: [(path("AllUsers")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("UserName")?, path("ServiceSpecificCredentialId")?])),
        queryable: vec![path("UserName")?, path("ServiceSpecificCredentialId")?],
        // Explicit modeled pagination: the pinned operation omits the paginator trait.
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListSigningCertificates", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListUserPolicies", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("PolicyNames")?),
        bind: [(path("UserName")?, Binding::From { operation: name("ListUsers")?, path: path("UserName")? })].into_iter().collect(),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::Request(path("UserName")?), IdentityField::Selected(path("$")?)])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListUserTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires owner identity from ListUsers.UserName and paginated tag merge into that user.".to_owned()),
        ..empty()
    }),
("ListUsers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Users")?),
        page_size: Some(1000),
        exclude_from_hash: vec![path("PasswordLastUsed")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListVirtualMFADevices", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("PutAccountProperties", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("PutGroupPolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("PutRolePermissionsBoundary", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("PutRolePolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("PutUserPermissionsBoundary", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("PutUserPolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RejectDelegationRequest", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemoveClientIDFromOpenIDConnectProvider", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemoveRoleFromInstanceProfile", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemoveUserFromGroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ResetServiceSpecificCredential", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
