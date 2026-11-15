use crate::overlay::{NameError, OperationOverlay, OverlayClass};
use super::super::{empty};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("VerifyDnsConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeConnectionFunction", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Metadata only, not code, but its Identifier and Stage require a complete stage-aware ListConnectionFunctions source; that source has undocumented unfiltered-stage behavior.".to_owned()),
        ..empty()
    }),
("DescribeFunction", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns FunctionSummary already included by unfiltered ListFunctions; the extra ETag is revision metadata, not additional configuration.".to_owned()),
        ..empty()
    }),
("DescribeKeyValueStore", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns the same KeyValueStore metadata as ListKeyValueStores plus ETag; no store contents or additional configuration is needed.".to_owned()),
        ..empty()
    }),
("GetCachePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Custom CachePolicy including its configuration is already present in ListCachePolicies; Get adds only ETag. Managed policy enumeration remains reference data.".to_owned()),
        ..empty()
    }),
("GetCachePolicyConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CachePolicyConfig is already nested in ListCachePolicies.CachePolicyList.Items.CachePolicy; this config-only result lacks its own policy ID and adds only ETag.".to_owned()),
        ..empty()
    }),
("GetCloudFrontOriginAccessIdentity", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListCloudFrontOriginAccessIdentities already supplies Id, S3CanonicalUserId and Comment; Get adds creation CallerReference and ETag, not additional current configuration.".to_owned()),
        ..empty()
    }),
("GetCloudFrontOriginAccessIdentityConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Comment is already included in ListCloudFrontOriginAccessIdentities; config-only output adds creation CallerReference and ETag but has no identity ID.".to_owned()),
        ..empty()
    }),
("GetConnectionGroupByRoutingEndpoint", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("RoutingEndpoint lookup returns the same ConnectionGroup as the reviewed GetConnectionGroup bound by Id; no separate lookup is necessary.".to_owned()),
        ..empty()
    }),
("GetContinuousDeploymentPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Complete ContinuousDeploymentPolicyConfig is already returned by ListContinuousDeploymentPolicies; Get adds only ETag.".to_owned()),
        ..empty()
    }),
("GetContinuousDeploymentPolicyConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Config is already in ListContinuousDeploymentPolicies; config-only output lacks policy ID and adds only ETag.".to_owned()),
        ..empty()
    }),
("GetDistributionTenant", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Unlike ListDistributionTenants summaries, this result includes Parameters.Value. Safe collection requires reviewed pre-persistence projection of tenant parameter values, not hash exclusion.".to_owned()),
        ..empty()
    }),
("GetDistributionTenantByDomain", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Domain lookup returns DistributionTenant.Parameters.Value like GetDistributionTenant; do not persist unreviewed parameter values through an alternate lookup.".to_owned()),
        ..empty()
    }),
("GetFieldLevelEncryption", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListFieldLevelEncryptionConfigs already includes QueryArgProfileConfig and ContentTypeProfileConfig; Get adds creation CallerReference and ETag.".to_owned()),
        ..empty()
    }),
("GetFieldLevelEncryptionConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Config is already represented by ListFieldLevelEncryptionConfigs; config-only result has no encryption-config ID and adds creation CallerReference and ETag.".to_owned()),
        ..empty()
    }),
("GetFieldLevelEncryptionProfile", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListFieldLevelEncryptionProfiles already includes Name, EncryptionEntities and Comment; Get adds creation CallerReference and ETag, not key material.".to_owned()),
        ..empty()
    }),
("GetFieldLevelEncryptionProfileConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Profile config is already represented by ListFieldLevelEncryptionProfiles; config-only result lacks profile ID and adds creation CallerReference and ETag.".to_owned()),
        ..empty()
    }),
("GetInvalidation", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires DistributionId and invalidation Id. ListInvalidations cannot safely obtain a distribution source while ListDistributions exposes origin-header secrets; nested parent identity must survive invalidation enumeration.".to_owned()),
        ..empty()
    }),
("GetInvalidationForDistributionTenant", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires DistributionTenantId plus invalidation Id; ListInvalidationsForDistributionTenant results omit tenant identity. Generic request-context identity and correlated parent bindings are needed.".to_owned()),
        ..empty()
    }),
("GetKeyGroup", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListKeyGroups already includes KeyGroupConfig with public-key IDs, not encoded keys; Get adds only ETag.".to_owned()),
        ..empty()
    })
    ])
}
