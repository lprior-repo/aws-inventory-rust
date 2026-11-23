use super::super::{empty, name, path};
use crate::overlay::{
    Binding, FixedValue, Identity, IdentityField, ItemSelector, NameError, OperationOverlay,
    OverlayClass, Paginator, ResourceProjection, Scope, TagEndpoint, TagEndpointParent,
    TagEndpointRequest, TagEndpointSelector, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListGroupPolicies", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("PolicyNames")?),
        bind: [(path("GroupName")?, Binding::From { operation: name("ListGroups")?, path: path("GroupName")? })].into_iter().collect(),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::Request(path("GroupName")?), IdentityField::Selected(path("$")?)])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Groups")?),
        page_size: Some(1000),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListGroupsForUser", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Groups has valid group ARNs but omits the requested UserName. Requires membership edge extraction with input user context; independently storing groups discards the operation purpose.".to_owned()),
        ..empty()
    }),
("ListInstanceProfileTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires owner identity from ListInstanceProfiles.InstanceProfileName and paginated tag merge into the instance-profile record, not globally keyed tag resources.".to_owned()),
        ..empty()
    }),
("ListInstanceProfiles", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("InstanceProfiles")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        exclude_response_members: vec![path("InstanceProfiles.Roles.AssumeRolePolicyDocument")?],
        page_size: Some(1000),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListInstanceProfilesForRole", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("InstanceProfiles")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        exclude_response_members: vec![path("InstanceProfiles.Roles.AssumeRolePolicyDocument")?],
        bind: [(path("RoleName")?, Binding::From { operation: name("ListRoles")?, path: path("RoleName")? })].into_iter().collect(),
        page_size: Some(1000),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListMFADeviceTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires owner identity from ListMFADevices.SerialNumber and paginated parent-tag merge; the API applies to virtual devices, requiring device-kind discrimination before scheduling.".to_owned()),
        ..empty()
    }),
("ListMFADevices", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("MFADevices")?),
        page_size: Some(1000),
        bind: [(path("UserName")?, Binding::From { operation: name("ListUsers")?, path: path("UserName")? })].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("SerialNumber")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListOpenIDConnectProviderTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires input owner ARN from ListOpenIDConnectProviders.Arn and paginated parent-tag merge rather than tag resources with colliding Key identities.".to_owned()),
        ..empty()
    }),
("ListOpenIDConnectProviders", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("OpenIDConnectProviderList")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListOrganizationsFeatures", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("EnabledFeatures is organization configuration and OrganizationId identifies the organization, but organization-scoped metadata merge is absent. Must preserve the organization/account relationship rather than create an unrelated feature resource.".to_owned()),
        ..empty()
    }),
("ListPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Policies")?),
        page_size: Some(1000),
        exclude_from_hash: vec![path("AttachmentCount")?, path("PermissionsBoundaryUsageCount")?],
        fixed: [(path("Scope")?, FixedValue::String("Local".to_owned())), (path("OnlyAttached")?, FixedValue::Boolean(false))].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListPoliciesGrantingServiceAccess", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Arn requires fan-in from users, groups and roles; required ServiceNamespaces has no read-only complete namespace discovery for this analysis. Nested PoliciesGrantingServiceAccess requires principal/service-qualified relationships and mixed inline/managed policy identities.".to_owned()),
        ..empty()
    }),
("ListPolicyTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires owner ARN from ListPolicies.Arn and paginated tag merge into the local managed-policy record.".to_owned()),
        ..empty()
    }),
("ListPolicyVersions", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Versions")?),
        bind: [(path("PolicyArn")?, Binding::From { operation: name("ListPolicies")?, path: path("Arn")? })].into_iter().collect(),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::Request(path("PolicyArn")?), IdentityField::Selected(path("VersionId")?)])),
        // The pinned documentation explicitly excludes Document from this list response.
        queryable: vec![path("VersionId")?],
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListRolePolicies", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("PolicyNames")?),
        bind: [(path("RoleName")?, Binding::From { operation: name("ListRoles")?, path: path("RoleName")? })].into_iter().collect(),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::Request(path("RoleName")?), IdentityField::Selected(path("$")?)])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListRoleTags", TagEndpoint {
        parents: vec![TagEndpointParent {
            operation: name("ListRoles")?,
            request: TagEndpointRequest::Member { input: path("RoleName")?, parent: path("RoleName")? },
        }],
        selector: TagEndpointSelector::List { path: path("Tags")?, key: path("Key")?, value: path("Value")? },
    }.operation(Scope::Account, Paginator::Tokens {
        pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }],
        truncated: Some(path("IsTruncated")?),
    })?),
("ListRoles", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Roles")?),
        projection: Some(ResourceProjection::IamTrustSummary(path("AssumeRolePolicyDocument")?)),
        exclude_from_hash: vec![path("RoleLastUsed")?],
        queryable: vec![path("Arn")?, path("RoleName")?, path("PermissionsBoundary.PermissionsBoundaryArn")?],
        page_size: Some(1000),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("Marker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListSAMLProviderTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Tags contains only Key and Value. Requires owner ARN from ListSAMLProviders.Arn and paginated parent-tag merge.".to_owned()),
        ..empty()
    }),
("ListSAMLProviders", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("SAMLProviderList")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    })
    ])
}
