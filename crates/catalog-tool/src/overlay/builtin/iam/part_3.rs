use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GenerateOrganizationsAccessReport", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("GenerateServiceLastAccessedDetails", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("GetAccessKeyLastUsed", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("The output omits the required AccessKeyId identity. Bind AccessKeyId from ListAccessKeys.AccessKeyId after shared input-context identity and parent metadata merge are available; exclude AccessKeyLastUsed.LastUsedDate from configuration hashing.".to_owned()),
        ..empty()
    }),
("GetAccountAuthorizationDetails", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("UserDetailList, GroupDetailList, RoleDetailList and Policies require multiple typed item selections and relationship extraction; embedded URL-encoded inline, managed and trust policies require bounded model-driven decoding and sanitization before adapter-equivalent publication.".to_owned()),
        ..empty()
    }),
("GetAccountPasswordPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PasswordPolicy contains safe account configuration but no account identifier. Requires context-account singleton identity and account-record enrichment, neither expressible by output-only identity paths.".to_owned()),
        ..empty()
    }),
("GetAccountProperties", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Properties is a map of Namespace/PropertyName to values, with no account identifier. Requires context-account singleton identity and preservation of map keys for typed account configuration.".to_owned()),
        ..empty()
    }),
("GetAccountSummary", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("SummaryMap contains mutable usage counts and quotas, not an output account identifier. Requires context-account identity and per-counter hash exclusions before account metadata enrichment.".to_owned()),
        ..empty()
    }),
("GetContextKeysForCustomPolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetContextKeysForPrincipalPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PolicySourceArn must fan in user, group and role ARN sources; ContextKeyNames is a scalar list requiring principal-qualified scalar identity. One-source bindings and output-only identity cannot encode that analysis result.".to_owned()),
        ..empty()
    }),
("GetCredentialReport", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetDelegationRequest", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("DelegationRequest includes RequestMessage, Notes and RejectionReason user-authored message content; no response projection exists to persist only delegation configuration metadata.".to_owned()),
        ..empty()
    }),
("GetGroup", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Group is sibling metadata and Users is a paginated member list. Requires group-user membership edges and preservation of the input/group identity on each selected user; selecting Users alone loses the membership relationship.".to_owned()),
        ..empty()
    }),
("GetGroupPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GroupName and PolicyName identify the inline configuration, but PolicyDocument is URL-encoded policy text. Requires bounded policy decoding and reviewed relation extraction rather than retaining opaque runtime JSON policy strings.".to_owned()),
        ..empty()
    }),
("GetHumanReadableSummary", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetInstanceProfile", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("InstanceProfile.Roles embeds Role.AssumeRolePolicyDocument. Requires bounded URL-encoded trust-policy sanitization matching the existing adapter and reviewed instance-profile-to-role edges before generic publication.".to_owned()),
        ..empty()
    }),
("GetLoginProfile", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("$")?),
        bind: [(path("UserName")?, Binding::From { operation: name("ListUsers")?, path: path("UserName")? })].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("LoginProfile.UserName")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetMFADevice", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GetMFADevice accepts only FIDO security-key ARNs, while ListMFADevices.SerialNumber also includes virtual-device ARNs and hardware serial numbers. Requires a generic device-kind eligibility predicate before binding SerialNumber and UserName from ListMFADevices; unfiltered expansion is invalid.".to_owned()),
        ..empty()
    }),
("GetOpenIDConnectProvider", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Url, ClientIDList, ThumbprintList and Tags are safe configuration, but the output omits OpenIDConnectProviderArn. Requires identity from bound input ListOpenIDConnectProviders.Arn and parent-record merge.".to_owned()),
        ..empty()
    }),
("GetOrganizationsAccessReport", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("JobId is returned only by mutating GenerateOrganizationsAccessReport. No safe account enumeration discovers existing report jobs; report access history is not configurable-resource inventory.".to_owned()),
        ..empty()
    }),
("GetOutboundWebIdentityFederationInfo", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("IssuerIdentifier and JwtVendingEnabled are account configuration, not a guaranteed account identity; disabled accounts may omit the issuer. Requires context-account singleton identity.".to_owned()),
        ..empty()
    })
    ])
}
