use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DisassociateAccessPolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DisassociateIdentityProviderConfig", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ListAccessEntries", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET owned access-entry principal ARNs in accessEntries[], token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $. Selected principal ARN alone conflates entries for the same principal in different clusters; need parent-qualified identity and retained cluster context for DescribeAccessEntry, not the IAM principal's identity as the access entry.".to_owned()),
        ..empty()
    }),
("ListAccessPolicies", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("ListAddons", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET installed add-on names in addons[], token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $. Selected add-on name is a string without cluster context and is reusable across clusters; need parent-qualified identity and retained cluster context for DescribeAddon. EKS Addon mapping is absent.".to_owned()),
        ..empty()
    }),
("ListAssociatedAccessPolicies", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET owned access-policy associations, associatedAccessPolicies[], token nextToken -> nextToken, maxResults (1..100). Required clusterName/principalArn can bind together from DescribeAccessEntry.accessEntry, but that parent is blocked. Selected association has policyArn/accessScope/timestamps but no owning cluster or principal; policyArn identifies reusable reference policy, not its association. Need request-qualified association identity and reviewed kind/relations.".to_owned()),
        ..empty()
    }),
("ListCapabilities", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("capabilities")?),
        page_size: Some(5),
        bind: [(path("clusterName")?, Binding::From { operation: name("ListClusters")?, path: path("$")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListCertificateAuthorities", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET cluster CA summaries in certificateAuthorities[], token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $. Summary id has no ARN or clusterName; need cluster-qualified identity and retained cluster request context for DescribeCertificateAuthority. Public CA metadata is not a private-key operation; reviewed API-only CA kind/relations are absent.".to_owned()),
        ..empty()
    }),
("ListClusters", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("clusters")?),
        page_size: Some(5),
        fixed: [(path("include")?, FixedValue::Strings(vec!["all".to_owned()]))].into_iter().collect(),
        identity: Some(Identity::Template("arn:{partition}:eks:{region}:{account}:cluster/{$}".to_owned())),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListEksAnywhereSubscriptions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("ListFargateProfiles", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET owned Fargate profile names in fargateProfileNames[], token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $. Selected profile name is a string without owning cluster context and is reusable across clusters; need parent-qualified identity and retained cluster context for DescribeFargateProfile. EKS FargateProfile mapping is absent.".to_owned()),
        ..empty()
    }),
("ListIdentityProviderConfigs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET owned identity-provider config summaries in identityProviderConfigs[], token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $. Items contain type/name only; type/name is not unique across clusters, so need request-qualified identity and correlated parent context for DescribeIdentityProviderConfig.identityProviderConfig.type/name. AWS::EKS::IdentityProviderConfig mapping is absent.".to_owned()),
        ..empty()
    }),
("ListInsights", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Read-only modeled POST /clusters/{clusterName}/insights lists configuration/upgrade findings in insights[]; default filter covers every category/status. Token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $; selected id lacks owning cluster context. Need cluster-qualified identity/correlated DescribeInsight expansion and a reviewed API-only kind. No StartInsightsRefresh call is authorized.".to_owned()),
        ..empty()
    }),
("ListNodegroups", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET managed node-group names in nodegroups[]; self-managed groups are explicitly not listed. Token nextToken -> nextToken, maxResults (1..100). Required clusterName can bind from ListClusters scalar item $. Selected name is reusable across clusters and lacks cluster context. Need parent-qualified identity and retained cluster context for DescribeNodegroup, not a Cartesian join of cluster and node-group names. EKS Nodegroup mapping is absent.".to_owned()),
        ..empty()
    }),
("ListPodIdentityAssociations", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("associations")?),
        page_size: Some(5),
        bind: [(path("clusterName")?, Binding::From { operation: name("ListClusters")?, path: path("$")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("associationArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET resource tags; required resourceArn, output root has only tags map, no response ARN or stable per-resource identity, no paginator. Tags are inline on modeled resource descriptions, so do not create standalone tag-value identities. Need explicit multi-resource ARN source selection and request-context enrichment/deduplication; current overlay represents neither a polymorphic tag enrichment nor inherited identity.".to_owned()),
        ..empty()
    }),
("ListUpdates", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET update IDs in updateIds[], token nextToken -> nextToken, maxResults (1..100). Required name and optional nodegroupName/addonName/capabilityName select distinct resource histories. Scalar ID can be selected with $, but selected items omit all resource context. Need request-qualified identity and correlated multi-variant parent request provenance; a single cluster-only plan would omit node-group/add-on/capability histories.".to_owned()),
        ..empty()
    }),
("RegisterCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("StartInsightsRefresh", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("TagResource", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
