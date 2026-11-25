use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetInstanceTpmEkPub", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires InstanceId, KeyType and KeyFormat; returns public rather than private key material, but full RSA/ECC and encoding coverage requires enum-input fanout and stable instance/key-type identity.".to_owned()),
        ..empty()
    }),
("GetInstanceTypesFromInstanceRequirements", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetInstanceUefiData", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetIpamAddressHistory", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires Cidr and IpamScopeId; a complete CIDR query domain and temporal HistoryRecords[] identity are not available from a single parent binding.".to_owned()),
        ..empty()
    }),
("GetIpamDiscoveredAccounts", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires IpamResourceDiscoveryId plus DiscoveryRegion; DiscoveryRegion must fan out the discovery operating regions, not blindly reuse request region, and child rows omit the discovery parent.".to_owned()),
        ..empty()
    }),
("GetIpamDiscoveredPublicAddresses", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires IpamResourceDiscoveryId plus AddressRegion; needs operating-region fanout and ownership propagation from AddressOwnerId, not treating discovered remote addresses as request-account resources.".to_owned()),
        ..empty()
    }),
("GetIpamDiscoveredResourceCidrs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires IpamResourceDiscoveryId plus ResourceRegion; needs operating-region fanout and ownership propagation from ResourceOwnerId with discovery-parent context.".to_owned()),
        ..empty()
    }),
("GetIpamDiscoveredRoutes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires IpamResourceDiscoveryId plus ResourceRegion; requires operating-region fanout, route-destination identity and discovered owner propagation.".to_owned()),
        ..empty()
    }),
("GetIpamInternetRegistryAssociationAsns", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required IpamInternetRegistryAssociationId can bind from DescribeIpamInternetRegistryAssociations but IpamInternetRegistryAssociationAsns[] omits the parent association and requires ASN identity compatible with long scalars.".to_owned()),
        ..empty()
    }),
("GetIpamInternetRegistryAssociationCidrs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required association ID has a source in DescribeIpamInternetRegistryAssociations, but IpamInternetRegistryAssociationCidrs[].Cidr needs the parent association in its identity.".to_owned()),
        ..empty()
    }),
("GetIpamPolicyAllocationRules", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("IpamPolicyId has a parent in DescribeIpamPolicies, but IpamPolicyDocuments[] requires rule/document identity beyond the repeated policy ID to avoid collapsing multiple allocation rules.".to_owned()),
        ..empty()
    }),
("GetIpamPolicyOrganizationTargets", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("IpamPolicyId has a parent in DescribeIpamPolicies, but OrganizationTargets[].OrganizationTargetId omits the policy parent; requires parent-aware association identity.".to_owned()),
        ..empty()
    }),
("GetIpamPoolAllocations", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPoolAllocations")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamPoolAllocationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("IpamPoolId")?, Binding::From { operation: name("DescribeIpamPools")?, path: path("IpamPoolId")? })].into_iter().collect(),
        ..empty()
    }),
("GetIpamPoolCidrs", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPoolCidrs")?),
        identity: Some(Identity::Key(vec![path("IpamPoolCidrId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("IpamPoolId")?, Binding::From { operation: name("DescribeIpamPools")?, path: path("IpamPoolId")? })].into_iter().collect(),
        ..empty()
    }),
("GetIpamPrefixListResolverRules", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required resolver ID has a source in DescribeIpamPrefixListResolvers but Rules[] can be static CIDR or IPAM-scope selections; requires parent-context conditional rule identity.".to_owned()),
        ..empty()
    }),
("GetIpamPrefixListResolverVersionEntries", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires resolver ID and version; Entries[].Cidr omits both and needs resolver/version context plus dependent version enumeration.".to_owned()),
        ..empty()
    }),
("GetIpamPrefixListResolverVersions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required resolver ID has a source in DescribeIpamPrefixListResolvers, but version rows omit it; needs resolver-scoped version identity rather than global version numbers.".to_owned()),
        ..empty()
    }),
("GetIpamResourceCidrs", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamResourceCidrs")?),
        identity: Some(Identity::Key(vec![path("IpamScopeId")?, path("ResourceRegion")?, path("ResourceOwnerId")?, path("ResourceId")?, path("ResourceCidr")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("IpamScopeId")?, Binding::From { operation: name("DescribeIpamScopes")?, path: path("IpamScopeId")? })].into_iter().collect(),
        ..empty()
    }),
("GetIpamRouteOriginAuthorizations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required registry association ID has a source, but IpamRouteOriginAuthorizations[] omits that parent; CIDR/ASN authorizations need parent-aware composite identity.".to_owned()),
        ..empty()
    }),
("GetIpamRouteProtectionFindings", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required IpamId has a source in DescribeIpams; RouteProtectionFindings[] requires remote ResourceOwnerId/ResourceRegion identity and finding-specific composite fields rather than local ownership.".to_owned()),
        ..empty()
    })
    ])
}
