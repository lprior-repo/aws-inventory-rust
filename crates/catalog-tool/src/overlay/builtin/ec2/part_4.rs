use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeInstanceSqlHaHistoryStates", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Instances[] repeats InstanceId over historical LastUpdatedTime states; using only InstanceId would collapse history and including mutable state would manufacture resource identities.".to_owned()),
        ..empty()
    }),
("DescribeInstanceSqlHaStates", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Instances")?),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInstanceStatus", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InstanceStatuses")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true)), (path("IncludeAllInstances")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInstanceTopology", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Instances")?),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInstanceTypeOfferings", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeIpamByoasn", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Byoasns")?),
        identity: Some(Identity::Key(vec![path("IpamId")?, path("Asn")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamExternalResourceVerificationTokens", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeIpamInternetRegistryAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamInternetRegistryAssociations")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamInternetRegistryAssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPolicies")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamPolicyId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamPoolAllocations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPoolAllocations")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamPoolAllocationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamPools", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPools")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamPoolId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamPrefixListResolverTargets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPrefixListResolverTargets")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamPrefixListResolverTargetId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamPrefixListResolvers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamPrefixListResolvers")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamPrefixListResolverId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamResourceDiscoveries", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamResourceDiscoveries")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamResourceDiscoveryId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamResourceDiscoveryAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamResourceDiscoveryAssociations")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamResourceDiscoveryAssociationId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpamScopes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IpamScopes")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamScopeId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpams", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Ipams")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("IpamId")?])),
        page_size: Some(100),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIpv6Pools", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Ipv6Pools")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("PoolId")?])),
        page_size: Some(10),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeKeyPairs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("KeyPairs")?),
        tags: Some(path("Tags")?),
        fixed: [(path("IncludePublicKey")?, FixedValue::Boolean(false))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("KeyPairId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeLaunchTemplateVersions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    })
    ])
}
