use crate::overlay::{NameError, OperationOverlay, Binding, BindingTransform, FixedValue, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path, name};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetTrafficPolicyInstance", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrafficPolicyInstance")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HostedZoneId")?, path("TrafficPolicyId")?, path("TrafficPolicyVersion")?, path("TrafficPolicyType")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("Id")?, Binding::From { operation: name("ListTrafficPolicyInstances")?, path: path("Id")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetTrafficPolicyInstanceCount", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TrafficPolicyInstanceCount is an account aggregate without stable account identity in output. Runtime compiler rejects contextual identity templates; this cannot stand in for enumerating policy instances.".to_owned()),
        ..empty()
    }),
("ListCidrBlocks", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CollectionId can bind ListCidrCollections.Id and NextToken pagination is modeled, but CidrBlocks items contain only CidrBlock/LocationName. Need parent collection identity to prevent identical blocks in different collections colliding; LocationName is optional filtering, not an account-global key.".to_owned()),
        ..empty()
    }),
("ListCidrLocations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CollectionId can bind ListCidrCollections.Id and NextToken pagination is modeled, but CidrLocations items contain only LocationName. Need parent-input collection identity and relation; names including the default location are not globally unique.".to_owned()),
        ..empty()
    }),
("ListGeoLocations", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("ListHostedZonesByName", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("HostedZones")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("Name")?, path("Config.PrivateZone")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        fixed: [(path("MaxItems")?, FixedValue::Integer(5))].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("DNSName")?, output: path("NextDNSName")?, optional: false }, TokenPair { input: path("HostedZoneId")?, output: path("NextHostedZoneId")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListHostedZonesByVPC", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required VPCId/VPCRegion need correlated cross-service VPC discovery and owner-aware external-zone identities. NextToken input/output pagination is reviewed, but same-service bind syntax cannot enumerate VPCs across regions. Documentation excludes associations through Route 53 Profiles; do not claim those edges.".to_owned()),
        ..empty()
    }),
("ListResourceRecordSetProtections", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("HostedZoneId can bind ListHostedZones.Id and NextToken pagination is modeled, but RecordName/RecordType protections omit zone ID. Need parent-input identity and same-zone record relation; optional StartRecordName/StartRecordType must remain unset for complete discovery.".to_owned()),
        ..empty()
    }),
("ListResourceRecordSets", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires StartRecordName/NextRecordName and StartRecordType/NextRecordType as required pairs, with StartRecordIdentifier/NextRecordIdentifier optional=true, gated by IsTruncated. Pagination capability alone is insufficient: items omit zone ID and SetIdentifier is optional, requiring parent-input identity and presence-aware keys. Raw TXT/SPF RDATA can contain arbitrary content; no output projection replicates adapter safe_record_values filtering.".to_owned()),
        ..empty()
    }),
("ListReusableDelegationSets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("DelegationSets")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("NameServers")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        fixed: [(path("MaxItems")?, FixedValue::Integer(5))].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ResourceTagSet has ResourceType/ResourceId/Tags, but complete coverage needs two correlated variants: ResourceType=hostedzone with zone ID and ResourceType=healthcheck with health-check ID. Schema cannot union bindings or normalize optional /hostedzone/ prefixes before TagResourceId. A single fixed variant silently omits the other resource family.".to_owned()),
        ..empty()
    }),
("ListTagsForResources", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Read-only POST, not a mutation. Requires typed batches of at most ten ResourceIds, separately for hostedzone and healthcheck. Need correlated operation variants, ID prefix normalization, list materialization and a verified batch replacement runtime path; batch metadata alone must not enable incomplete mixed-type requests.".to_owned()),
        ..empty()
    }),
("ListTrafficPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrafficPolicySummaries")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("Name")?, path("Type")?, path("LatestVersion")?, path("TrafficPolicyCount")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        fixed: [(path("MaxItems")?, FixedValue::Integer(5))].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("TrafficPolicyIdMarker")?, output: path("TrafficPolicyIdMarker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListTrafficPolicyInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrafficPolicyInstances")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HostedZoneId")?, path("Name")?, path("TrafficPolicyId")?, path("TrafficPolicyVersion")?, path("TrafficPolicyType")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        fixed: [(path("MaxItems")?, FixedValue::Integer(5))].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("HostedZoneIdMarker")?, output: path("HostedZoneIdMarker")?, optional: false }, TokenPair { input: path("TrafficPolicyInstanceNameMarker")?, output: path("TrafficPolicyInstanceNameMarker")?, optional: false }, TokenPair { input: path("TrafficPolicyInstanceTypeMarker")?, output: path("TrafficPolicyInstanceTypeMarker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListTrafficPolicyInstancesByHostedZone", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrafficPolicyInstances")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HostedZoneId")?, path("Name")?, path("TrafficPolicyId")?, path("TrafficPolicyVersion")?, path("TrafficPolicyType")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("HostedZoneId")?, Binding::Transformed {
            binding: Box::new(Binding::From { operation: name("ListHostedZones")?, path: path("Id")? }),
            transform: BindingTransform::StripPrefix("/hostedzone/".to_owned()),
        })].into_iter().collect(),
        fixed: [(path("MaxItems")?, FixedValue::Integer(5))].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![
            TokenPair { input: path("TrafficPolicyInstanceNameMarker")?, output: path("TrafficPolicyInstanceNameMarker")?, optional: false },
            TokenPair { input: path("TrafficPolicyInstanceTypeMarker")?, output: path("TrafficPolicyInstanceTypeMarker")?, optional: false },
        ], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListTrafficPolicyInstancesByPolicy", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrafficPolicyInstances")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HostedZoneId")?, path("Name")?, path("TrafficPolicyId")?, path("TrafficPolicyVersion")?, path("TrafficPolicyType")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("TrafficPolicyId")?, Binding::From { operation: name("ListTrafficPolicyInstances")?, path: path("TrafficPolicyId")? }), (path("TrafficPolicyVersion")?, Binding::From { operation: name("ListTrafficPolicyInstances")?, path: path("TrafficPolicyVersion")? })].into_iter().collect(),
        fixed: [(path("MaxItems")?, FixedValue::Integer(5))].into_iter().collect(),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("HostedZoneIdMarker")?, output: path("HostedZoneIdMarker")?, optional: false }, TokenPair { input: path("TrafficPolicyInstanceNameMarker")?, output: path("TrafficPolicyInstanceNameMarker")?, optional: false }, TokenPair { input: path("TrafficPolicyInstanceTypeMarker")?, output: path("TrafficPolicyInstanceTypeMarker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("ListTrafficPolicyVersions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required Id can bind ListTrafficPolicies.Id. TrafficPolicyVersionMarker input/output is a string cursor gated by IsTruncated, not Version arithmetic; Id+Version is stable. TrafficPolicies items also contain opaque Document; need the same reviewed generic safe-document capability as GetTrafficPolicy before exposing or parsing its embedded JSON.".to_owned()),
        ..empty()
    }),
("ListVPCAssociationAuthorizations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("HostedZoneId can bind ListHostedZones.Id and NextToken input/output pagination is reviewed, but VPCs items need parent-zone identity and correlated VPCRegion/VPCId. An authorization is permission for association, not evidence of an existing IN_VPC association.".to_owned()),
        ..empty()
    }),
("TestDNSAnswer", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ActivateKeySigningKey", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
