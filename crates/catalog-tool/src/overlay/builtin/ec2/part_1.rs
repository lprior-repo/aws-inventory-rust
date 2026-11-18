use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeApplicationStatusCheckAssociations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Associations[] combines ApplicationStatusCheckId, AssociationType, Key and Value; AssociationType is enum-valued and identity validation currently accepts only string/integer/boolean.".to_owned()),
        ..empty()
    }),
("DescribeApplicationStatusChecks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ApplicationStatusChecks")?),
        identity: Some(Identity::Key(vec![path("ApplicationStatusCheckId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeAvailabilityZones", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeAwsNetworkPerformanceMetricSubscriptions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Subscriptions[] requires Source, Destination, Metric, Statistic and Period identity; Metric/Statistic/Period are enum shapes rejected by current identity validation.".to_owned()),
        ..empty()
    }),
("DescribeBundleTasks", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeByoipCidrs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ByoipCidrs")?),
        page_size: Some(100),
        identity: Some(Identity::Key(vec![path("Cidr")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityBlockExtensionHistory", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CapacityBlockExtensions[] is extension history identified by reservation/offering and CapacityBlockExtensionPurchaseDate; identity validation rejects timestamp keys and no unique extension ID is returned.".to_owned()),
        ..empty()
    }),
("DescribeCapacityBlockExtensionOfferings", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeCapacityBlockOfferings", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeCapacityBlockStatus", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityBlockStatuses")?),
        identity: Some(Identity::Key(vec![path("CapacityBlockId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityBlocks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityBlocks")?),
        identity: Some(Identity::Key(vec![path("CapacityBlockId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityManagerDataExports", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityManagerDataExports")?),
        identity: Some(Identity::Key(vec![path("CapacityManagerDataExportId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityReservationBillingRequests", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required Role distinguishes odcr-owner and unused-reservation-billing-owner views; one fixed Role silently drops the other view and overlay cannot fan out an enum input.".to_owned()),
        ..empty()
    }),
("DescribeCapacityReservationCancellationQuotes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityReservationCancellationQuotes")?),
        identity: Some(Identity::Key(vec![path("CapacityReservationCancellationQuoteId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityReservationDateChangeQuotes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityReservationModificationQuotes")?),
        identity: Some(Identity::Key(vec![path("CapacityReservationModificationQuoteId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityReservationFleets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityReservationFleets")?),
        identity: Some(Identity::Key(vec![path("CapacityReservationFleetId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityReservationTopology", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityReservations")?),
        identity: Some(Identity::Key(vec![path("CapacityReservationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityReservations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityReservations")?),
        identity: Some(Identity::Key(vec![path("CapacityReservationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCarrierGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CarrierGateways")?),
        identity: Some(Identity::Key(vec![path("CarrierGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeClassicLinkInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Instances")?),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    })
    ])
}
