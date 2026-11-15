use crate::overlay::{NameError, OperationOverlay, Binding, Identity, ItemSelector, OverlayClass, Paginator, Scope};
use super::super::{empty, path, name};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DeleteFargateProfile", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteNodegroup", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeletePodIdentityAssociation", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeregisterCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DescribeAccessEntry", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET cluster access-entry configuration; output root $, identity accessEntry.accessEntryArn, no paginator. Required principalArn can bind from ListAccessEntries scalar item $, but that item omits required clusterName. Need correlated originating cluster context, not independent principal/cluster bindings. ListAccessEntries also lacks cluster-qualified item identity. EKS AccessEntry kind/relation mappings are absent.".to_owned()),
        ..empty()
    }),
("DescribeAddon", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET installed add-on configuration; output root $, identity addon.addonArn, no paginator. Required addonName can bind from ListAddons scalar item $, but that item omits required clusterName. Need correlated originating cluster context rather than independently joining cluster and add-on names. ConfigurationValues is supplied add-on configuration, not a download endpoint. EKS Addon kind/relation mappings are absent.".to_owned()),
        ..empty()
    }),
("DescribeAddonConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeAddonVersions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeCapability", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET managed capability configuration; output root $, identity capability.arn, no paginator. Required capabilityName comes from ListCapabilities.capabilities[].capabilityName, but that item lacks required clusterName. Need correlated originating cluster request context rather than independently joining capability names and cluster names. EKS Capability detail kind/relation mappings are absent.".to_owned()),
        ..empty()
    }),
("DescribeCertificateAuthority", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET CA metadata and explicitly public certificate data, not a private key. Output root $, no paginator; certificateAuthority.id requires cluster-qualified identity. ListCertificateAuthorities.certificateAuthorities[].id supplies certificateAuthorityId but lacks required clusterName. Need request-context identity and correlated parent-cluster provenance, plus reviewed CA kind/relations.".to_owned()),
        ..empty()
    }),
("DescribeCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeClusterVersions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeEksAnywhereSubscription", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeFargateProfile", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET Fargate scheduling configuration; output root $, identity fargateProfile.fargateProfileArn, no paginator. Required fargateProfileName can bind from ListFargateProfiles scalar item $, but that item omits required clusterName. Need correlated originating cluster context rather than independently joining cluster and profile names. EKS FargateProfile kind/relation mappings are absent.".to_owned()),
        ..empty()
    }),
("DescribeIdentityProviderConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Read-only modeled POST /clusters/{clusterName}/identity-provider-configs/describe; output root $, identity identityProviderConfig.oidc.identityProviderConfigArn, no paginator. Required identityProviderConfig.type/name can bind separately from ListIdentityProviderConfigs item type/name, but the item lacks required clusterName. Need correlated originating cluster context, plus AWS::EKS::IdentityProviderConfig kind/relations.".to_owned()),
        ..empty()
    }),
("DescribeInsight", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET cluster configuration/upgrade insight; output root $, no paginator. ListInsights.insights[].id supplies required id but not required clusterName. Insight has no ARN or clusterName, so insight.id needs a cluster-qualified identity from request provenance. Need correlated cluster context across ListInsights/DescribeInsight and an individually reviewed API-only insight kind.".to_owned()),
        ..empty()
    }),
("DescribeInsightsRefresh", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET latest refresh status only; does not initiate refresh. Output message/status/startedAt/endedAt has no ARN, clusterName, or stable item identity. Requires parent-cluster identity/enrichment from required clusterName request context; do not identify a refresh resource by status or timestamps. An enrichment-only plan with inherited request identity is not represented by the current overlay.".to_owned()),
        ..empty()
    }),
("DescribeNodegroup", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET managed node-group configuration; output root $, identity nodegroup.nodegroupArn, no paginator. Required nodegroupName can bind from ListNodegroups scalar item $, but that item omits required clusterName. Need correlated originating cluster context, not independent cluster and node-group bindings. EKS Nodegroup kind/relations are absent; preserve SDK IN_CLUSTER, ASSUMES_ROLE and USES_SUBNET parity before cutover.".to_owned()),
        ..empty()
    }),
("DescribePodIdentityAssociation", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("association")?),
        tags: Some(path("tags")?),
        bind: [(path("clusterName")?, Binding::From { operation: name("ListPodIdentityAssociations")?, path: path("clusterName")? }), (path("associationId")?, Binding::From { operation: name("ListPodIdentityAssociations")?, path: path("associationId")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("associationArn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeUpdate", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GET asynchronous resource update status/errors, not a mutation. Output root $, no paginator. Required name/updateId and optional nodegroupName/addonName/capabilityName must preserve the exact ListUpdates request context. update.id lacks resource context; request-qualified identity and correlated multi-variant parent context are unavailable. Cluster-only collection would silently omit child-resource update histories.".to_owned()),
        ..empty()
    })
    ])
}
