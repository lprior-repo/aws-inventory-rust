use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    ResourceProjection, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetPolicy", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Policy")?),
        exclude_from_hash: vec![path("AttachmentCount")?, path("PermissionsBoundaryUsageCount")?],
        tags: Some(path("Tags")?),
        bind: [(path("PolicyArn")?, Binding::From { operation: name("ListPolicies")?, path: path("Arn")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetPolicyVersion", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PolicyVersion.VersionId is only policy-local and the output omits PolicyArn. Requires correlated parent PolicyArn plus Versions.VersionId lineage, parent-qualified identity and bounded decoding of URL-encoded PolicyVersion.Document.".to_owned()),
        ..empty()
    }),
("GetRole", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Role")?),
        projection: Some(ResourceProjection::IamTrustSummary(path("AssumeRolePolicyDocument")?)),
        exclude_from_hash: vec![path("RoleLastUsed")?],
        tags: Some(path("Tags")?),
        queryable: vec![path("Arn")?, path("RoleName")?, path("PermissionsBoundary.PermissionsBoundaryArn")?],
        bind: [(path("RoleName")?, Binding::From { operation: name("ListRoles")?, path: path("RoleName")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetRolePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("RoleName and PolicyName identify inline configuration, but PolicyDocument is URL-encoded policy text. Requires bounded policy decoding and reviewed relation extraction rather than retaining opaque runtime JSON policy strings.".to_owned()),
        ..empty()
    }),
("GetRoleTemplateVersion", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TemplateArn can occur at Roles.SourceRoleTemplate.TemplateArn. RoleTemplateVersion embeds trust/inline policy templates; no approved omission or content-read capability exists.".to_owned()),
        ..empty()
    }),
("GetSAMLProvider", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Output omits SAMLProviderArn; requires parent input identity from ListSAMLProviders.Arn. SAMLMetadataDocument is uploaded XML configuration requiring reviewed bounded handling; PrivateKeyList contains KeyId and Timestamp metadata only, not private key bytes.".to_owned()),
        ..empty()
    }),
("GetSSHPublicKey", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetServerCertificate", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetServiceLastAccessedDetails", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required JobId comes only from mutating GenerateServiceLastAccessedDetails, with report ownership tied to its generating session. No read-only resource enumeration discovers these asynchronous access-history jobs.".to_owned()),
        ..empty()
    }),
("GetServiceLastAccessedDetailsWithEntities", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required JobId comes only from mutating GenerateServiceLastAccessedDetails and ServiceNamespace from that report. Requires session-correlated report job lineage; historical entity access is not configurable-resource enumeration.".to_owned()),
        ..empty()
    }),
("GetServiceLinkedRoleDeletionStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("DeletionTaskId comes only from mutating DeleteServiceLinkedRole. No read-only operation enumerates deletion tasks, so inventory cannot bind this deletion workflow input.".to_owned()),
        ..empty()
    }),
("GetUser", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("User")?),
        exclude_from_hash: vec![path("PasswordLastUsed")?],
        tags: Some(path("Tags")?),
        bind: [(path("UserName")?, Binding::From { operation: name("ListUsers")?, path: path("UserName")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetUserPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("UserName and PolicyName identify inline configuration, but PolicyDocument is URL-encoded policy text. Requires bounded policy decoding and reviewed relation extraction rather than retaining opaque runtime JSON policy strings.".to_owned()),
        ..empty()
    }),
("ListAccessKeys", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("AccessKeyMetadata")?),
        page_size: Some(1000),
        bind: [(path("UserName")?, Binding::From { operation: name("ListUsers")?, path: path("UserName")? })].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("AccessKeyId")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListAccountAliases", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("AccountAliases")?),
        identity: Some(Identity::Key(vec![path("$")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListAttachedGroupPolicies", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AttachedPolicies contains PolicyArn and PolicyName but omits GroupName. Requires parent group-to-policy attachment edges and parent-context retention; independent policy records lose the attachment semantics.".to_owned()),
        ..empty()
    }),
("ListAttachedRolePolicies", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AttachedPolicies contains PolicyArn and PolicyName but omits RoleName. Requires parent role-to-policy attachment edge extraction; independent policy records lose the attachment semantics.".to_owned()),
        ..empty()
    }),
("ListAttachedUserPolicies", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AttachedPolicies contains PolicyArn and PolicyName but omits UserName. Requires parent user-to-policy attachment edges and parent-context retention; independent policy records lose the attachment semantics.".to_owned()),
        ..empty()
    }),
("ListDelegationRequests", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("DelegationRequests uses full DelegationRequest values including RequestMessage, Notes and RejectionReason. Requires metadata-only response projection; overlay-only Marker pagination also needs compiler support and must honor lowercase isTruncated.".to_owned()),
        ..empty()
    }),
("ListEntitiesForPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PolicyGroups, PolicyUsers and PolicyRoles need three typed output selections and attachment-versus-permissions-boundary relationship semantics. Each child requires retained input PolicyArn; one item selector cannot represent this graph.".to_owned()),
        ..empty()
    })
    ])
}
