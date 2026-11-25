use super::super::empty;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetKeyGroupConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("KeyGroupConfig is already nested in ListKeyGroups; config-only result has no key-group ID and adds only ETag.".to_owned()),
        ..empty()
    }),
("GetMonitoringSubscription", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires DistributionId from a safe distribution inventory, currently secret-denied. Output MonitoringSubscription has no distribution identity; request-context identity is required.".to_owned()),
        ..empty()
    }),
("GetOriginAccessControl", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListOriginAccessControls already returns Name, Description, SigningProtocol, SigningBehavior and OriginAccessControlOriginType; Get adds only ETag and a wrapper.".to_owned()),
        ..empty()
    }),
("GetOriginAccessControlConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Configuration fields are already in ListOriginAccessControls; config-only result lacks control ID and adds only ETag.".to_owned()),
        ..empty()
    }),
("GetOriginRequestPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Custom OriginRequestPolicyConfig is already returned by ListOriginRequestPolicies; Get adds only ETag. Managed policies remain reference data.".to_owned()),
        ..empty()
    }),
("GetOriginRequestPolicyConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Config is already nested in ListOriginRequestPolicies; config-only result lacks policy ID and adds only ETag.".to_owned()),
        ..empty()
    }),
("GetRealtimeLogConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListRealtimeLogConfigs already returns ARN, Name, SamplingRate, EndPoints and Fields. This POST lookup adds no configuration and must not be confused with fetching log contents.".to_owned()),
        ..empty()
    }),
("GetResourcePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires ResourceArn of an owned shared resource. Complete inventory needs a union of eligible owned-resource parents and ownership filtering; current bind selects one operation and cannot express that union.".to_owned()),
        ..empty()
    }),
("GetResponseHeadersPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Custom ResponseHeadersPolicyConfig is already returned by ListResponseHeadersPolicies; Get adds only ETag. Managed policies remain reference data.".to_owned()),
        ..empty()
    }),
("GetResponseHeadersPolicyConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Config is already nested in ListResponseHeadersPolicies; config-only result lacks policy ID and adds only ETag.".to_owned()),
        ..empty()
    }),
("GetStreamingDistributionConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GetStreamingDistribution already returns StreamingDistributionConfig and an ARN-bound identity; config-only result adds no configuration and lacks its distribution ID.".to_owned()),
        ..empty()
    }),
("ListConnectionFunctions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ConnectionFunctionSummary is safe metadata, but optional Stage lacks documented unfiltered/default semantics. Need evidence that omission includes DEVELOPMENT and LIVE, or generic fixed-input variants with stage-aware identity.".to_owned()),
        ..empty()
    }),
("ListDistributionTenantsByCustomization", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Filtered WebACLArn or CertificateArn lookup returns the same DistributionTenantSummary records as unfiltered ListDistributionTenants; no additional inventory is produced.".to_owned()),
        ..empty()
    }),
("ListDistributionsByCachePolicyId", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns scalar DistributionIdList.Items, not full distributions. Generic scalar-item identity plus a relation to the bound CachePolicyId is needed; overlay identity paths cannot name the scalar item itself.".to_owned()),
        ..empty()
    }),
("ListDistributionsByKeyGroup", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns scalar DistributionIdList.Items; requires scalar-item identity and a relation retaining bound KeyGroupId rather than treating IDs as independent configuration.".to_owned()),
        ..empty()
    }),
("ListDistributionsByOriginRequestPolicyId", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns scalar DistributionIdList.Items; requires scalar-item identity and a relation retaining bound OriginRequestPolicyId. No origin-header secret is returned by this ID-only operation.".to_owned()),
        ..empty()
    }),
("ListDistributionsByOwnedResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("DistributionIdOwnerList returns Id and OwnerAccountId for a ResourceArn, including cross-account consumers. Need eligible owned-resource parent union and cross-account relation targeting; one parent binding would silently narrow coverage.".to_owned()),
        ..empty()
    }),
("ListDistributionsByResponseHeadersPolicyId", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns scalar DistributionIdList.Items; requires scalar-item identity and a relation retaining bound ResponseHeadersPolicyId.".to_owned()),
        ..empty()
    }),
("ListDistributionsByVpcOriginId", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns scalar DistributionIdList.Items; requires scalar-item identity and a relation retaining bound VpcOriginId, including resource-sharing account semantics.".to_owned()),
        ..empty()
    }),
("ListInvalidations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires DistributionId from currently secret-denied ListDistributions. Summary items contain Id, Status and CreateTime but omit parent distribution identity.".to_owned()),
        ..empty()
    })
    ])
}
