use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetCapacityManagerMetricDimensions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetCapacityManagerMonitoredTagKeys", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CapacityManagerTagKeys[] contains scalar strings; item identity currently requires structure-member paths and cannot identify a scalar item or account-scoped tag-key setting.".to_owned()),
        ..empty()
    }),
("GetCapacityManagerSupportabilityScores", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetCapacityReservationUsage", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required CapacityReservationId can bind from DescribeCapacityReservations, but paginated InstanceUsages[] omits the reservation ID; root selection would repeatedly overwrite the same reservation per page.".to_owned()),
        ..empty()
    }),
("GetClientVpnEndpointAuthorizationPolicy", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("ClientVpnEndpointId")?])),
        paginator: Some(Paginator::None),
        bind: [(path("ClientVpnEndpointId")?, Binding::From { operation: name("DescribeClientVpnEndpoints")?, path: path("ClientVpnEndpointId")? })].into_iter().collect(),
        ..empty()
    }),
("GetCoipPoolUsage", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required PoolId can bind from DescribeCoipPools, but CoipAddressUsages[] omits CoipPoolId; needs parent-aware usage identity without losing page data under a root singleton.".to_owned()),
        ..empty()
    }),
("GetConsoleOutput", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetConsoleScreenshot", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetDeclarativePoliciesReportSummary", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("ReportId")?])),
        paginator: Some(Paginator::None),
        bind: [(path("ReportId")?, Binding::From { operation: name("DescribeDeclarativePoliciesReports")?, path: path("ReportId")? })].into_iter().collect(),
        ..empty()
    }),
("GetDefaultCreditSpecification", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required InstanceFamily must derive from owned instance types; DescribeInstances is blocked on contextual flattening, and InstanceFamilyCreditSpecification lacks a stable context-free identity.".to_owned()),
        ..empty()
    }),
("GetEbsDefaultKmsKeyId", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("KmsKeyId is a mutable regional account setting, not the setting identity; requires account/region-context singleton identity so changing the key updates rather than replaces the resource.".to_owned()),
        ..empty()
    }),
("GetEbsEncryptionByDefault", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("EbsEncryptionByDefault and SseType are mutable regional account settings with no stable output identifier; requires account/region-context singleton identity.".to_owned()),
        ..empty()
    }),
("GetEnabledIpamPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("IpamPolicyId is absent when disabled and changes when policy changes; requires stable account/region settings identity rather than optional policy ID identity.".to_owned()),
        ..empty()
    }),
("GetFlowLogsIntegrationTemplate", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Generates a CloudFormation integration template from required FlowLogId, ConfigDeliveryS3DestinationArn and IntegrateServices; output Result is generated template content, not resource inventory, and no arbitrary delivery destination is supplied.".to_owned()),
        ..empty()
    }),
("GetGroupsForCapacityReservation", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CapacityReservationId can bind from DescribeCapacityReservations but CapacityReservationGroups[] omits it; needs parent-aware association identity to preserve the same group linked to multiple reservations.".to_owned()),
        ..empty()
    }),
("GetHostReservationPurchasePreview", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetImageAncestry", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required ImageId can bind from DescribeImages, but ImageAncestryEntries[] can refer to ancestors in SourceImageRegion; needs cross-region lineage identity and edge preservation rather than reclassifying ancestors as locally owned AMIs.".to_owned()),
        ..empty()
    }),
("GetImageBlockPublicAccessState", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ImageBlockPublicAccessState and ManagedBy are account/region settings without an output identifier; requires context-derived singleton identity.".to_owned()),
        ..empty()
    }),
("GetImageFindings", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required ImageId can bind from DescribeImages, but ImageFindings[] needs the parent image plus snapshot/finding identity; requires parent context and reviewed finding-version semantics.".to_owned()),
        ..empty()
    }),
("GetInstanceMetadataDefaults", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AccountLevel is mutable regional metadata-default configuration and has no stable output key; requires account/region-context singleton identity.".to_owned()),
        ..empty()
    })
    ])
}
